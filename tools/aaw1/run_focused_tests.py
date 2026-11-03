"""Compile actual diagnostic modules in a small native Rust harness.

Linux/WSL: python3 tools/aaw1/run_focused_tests.py
This executes real logger/capture/privacy/protobuf code, with only the MITM
transport context stubbed. It does not claim full-app or device coverage.
"""
import json
import os
from pathlib import Path
import re
import subprocess
import tempfile
import tomllib

ROOT = Path(__file__).resolve().parents[2]


def main():
    packages = tomllib.loads((ROOT / "Cargo.lock").read_text())["package"]
    def version(name, prefix=""):
        return next(p["version"] for p in packages if p["name"] == name and p["version"].startswith(prefix))
    with tempfile.TemporaryDirectory(prefix="aaw1-focused-") as temporary:
        harness = Path(temporary)
        (harness / "src").mkdir()
        (harness / "tests").mkdir()
        dependencies = {
            "protobuf": "3.7.1", "protobuf-json-mapping": "3.7.1",
            "base64": version("base64", "0.22"), "hex": version("hex"),
            "serde_json": version("serde_json"), "sha2": version("sha2", "0.10"),
            "log": version("log"), "tempfile": version("tempfile"),
        }
        cargo = '[package]\nname="aa-proxy-rs"\nversion="0.0.0"\nedition="2021"\n[dependencies]\n'
        cargo += ''.join(f'{name}="={v}"\n' for name, v in dependencies.items())
        cargo += f'tokio={{version="={version("tokio")}",features=["fs","io-util","sync","rt","macros","time"]}}\n'
        cargo += f'[build-dependencies]\nprotobuf-codegen="=3.7.1"\nprotoc-bin-vendored="={version("protoc-bin-vendored")}"\n'
        (harness / "Cargo.toml").write_text(cargo)
        proto = ROOT / "src/protos"
        inputs = ''.join(f'.input({json.dumps(str(proto / name))})' for name in ["protos.proto", "ev.proto", "WifiStartRequest.proto", "WifiInfoResponse.proto"])
        (harness / "build.rs").write_text('fn main(){protobuf_codegen::Codegen::new().protoc().protoc_path(&protoc_bin_vendored::protoc_bin_path().unwrap()).includes(&[' + json.dumps(str(proto)) + '])' + inputs + '.cargo_out_dir("protos").run_from_script();}')
        # Use the production enum's variants/order, without Linux Bluetooth imports.
        enum = re.search(r'pub enum HexdumpLevel \{(.*?)\n\}', (ROOT / "src/config_types.rs").read_text(), re.S).group(1).replace('#[default]', '')
        lib = 'pub mod config_types { #[derive(Debug,PartialEq,PartialOrd,Clone,Copy)] pub enum HexdumpLevel {' + enum + '} }\n'
        lib += '''pub mod mitm {
            include!(concat!(env!("OUT_DIR"), "/protos/mod.rs"));
            #[derive(Clone,Copy,Debug,PartialEq,Eq)] pub enum ProxyType { HeadUnit, MobileDevice }
            #[derive(Clone,Debug)] pub struct Packet { pub channel:u8, pub flags:u8, pub final_length:Option<u32>, pub payload:Vec<u8> }
            pub const FRAME_TYPE_FIRST:u8=1; pub const FRAME_TYPE_LAST:u8=2; pub const FRAME_TYPE_MASK:u8=3;
            pub type Result<T> = std::result::Result<T,Box<dyn std::error::Error+Send+Sync>>;
            pub fn get_name(p:ProxyType)->String { format!("{:?}",p) }
            pub struct ModifyContext { pub debug_channel_kinds:std::collections::HashMap<u8,crate::mitm_prettyprint::PacketDebugServiceKind>, pub pkt_debug_full_frames:crate::mitm_prettyprint::PktDebugFullFrameBuffers }
        }
        '''
        for name in ["sdr_text", "nav_sdr", "diagnostic_privacy", "debug_capture", "packet_debug_config", "companion_protocol", "mitm_prettyprint"]:
            lib += '#[path=' + json.dumps(str(ROOT / f"src/{name}.rs")) + f'] pub mod {name};\n'
        (harness / "src/lib.rs").write_text(lib)
        (harness / "tests/sdr_opaque_metadata.rs").write_bytes((ROOT / "tests/sdr_opaque_metadata.rs").read_bytes())
        environment = dict(os.environ)
        environment.setdefault("CARGO_TARGET_DIR", str(Path(tempfile.gettempdir()) / "aaw1-focused-target"))
        subprocess.run(["cargo", "test", "--all-targets"], cwd=harness, env=environment, check=True)


if __name__ == "__main__":
    main()

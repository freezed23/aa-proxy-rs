"""No Rust/product build: exercise the actual proto schema with synthetic data.

Requires binary wheels protobuf + grpcio-tools. Generates descriptors in a
temporary directory; does not run build.rs, cargo, CI or contact a device.
"""
import re
import subprocess
import tempfile
import unittest
from pathlib import Path

from google.protobuf import descriptor_pb2, descriptor_pool, message_factory
from google.protobuf.message import DecodeError, EncodeError
from grpc_tools import protoc

ROOT = Path(__file__).resolve().parents[2]
BASE = "e5b180f760ecb938a86a0c3b85f4eb50a60ac61a"
HU = ["make", "model", "year", "vehicle_id", "head_unit_make", "head_unit_model",
      "head_unit_software_build", "head_unit_software_version"]
CHANGED = {"HeadUnitInfo": HU, "ServiceDiscoveryResponse": HU + ["display_name"],
           "BluetoothService": ["car_address"],
           "VendorExtensionService": ["service_name", "package_white_list"],
           "WifiProjectionService": ["car_wifi_bssid"]}


def schema_pool(source, directory):
    (directory / "protos.proto").write_bytes(source)
    (directory / "ev.proto").write_bytes((ROOT / "src/protos/ev.proto").read_bytes())
    output = directory / "schema.pb"
    if protoc.main(["protoc", f"-I{directory}", "--include_imports",
                    f"--descriptor_set_out={output}", "protos.proto"]):
        raise RuntimeError("schema descriptor generation failed")
    descriptors = descriptor_pb2.FileDescriptorSet.FromString(output.read_bytes())
    pool = descriptor_pool.DescriptorPool()
    for descriptor in descriptors.file:
        pool.Add(descriptor)
    return pool, descriptors


def cls(pool, name="ServiceDiscoveryResponse"):
    return message_factory.GetMessageClass(pool.FindMessageTypeByName(name))


class SdrWireTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls_):
        cls_.temporary = tempfile.TemporaryDirectory()
        directory = Path(cls_.temporary.name)
        original = subprocess.check_output(["git", "show", f"{BASE}:src/protos/protos.proto"], cwd=ROOT)
        (directory / "old").mkdir()
        (directory / "new").mkdir()
        cls_.old, cls_.old_descriptor = schema_pool(original, directory / "old")
        cls_.new, cls_.new_descriptor = schema_pool((ROOT / "src/protos/protos.proto").read_bytes(), directory / "new")

    @classmethod
    def tearDownClass(cls_):
        cls_.temporary.cleanup()

    def test_only_21_field_types_change(self):
        before = descriptor_pb2.FileDescriptorSet()
        before.CopyFrom(self.old_descriptor)
        changed = []
        for file in before.file:
            for message in file.message_type:
                for field in message.field:
                    if field.name in CHANGED.get(message.name, []):
                        self.assertEqual(field.type, field.TYPE_STRING)
                        field.type = field.TYPE_BYTES
                        changed.append(f"{message.name}.{field.name}")
        self.assertEqual(len(changed), 21)
        self.assertEqual(before, self.new_descriptor)

    def test_sdr_reachable_messages_have_no_utf8_string_gate(self):
        seen = set()
        def visit(message):
            if message.full_name in seen:
                return
            seen.add(message.full_name)
            for field in message.fields:
                self.assertNotEqual(field.type, field.TYPE_STRING, field.full_name)
                if field.message_type:
                    visit(field.message_type)
        visit(self.new.FindMessageTypeByName("ServiceDiscoveryResponse"))

    def test_every_metadata_field_preserves_invalid_bytes(self):
        for message_name, fields in CHANGED.items():
            message = cls(self.new, message_name)()
            for name in fields:
                field = message.DESCRIPTOR.fields_by_name[name]
                if field.is_repeated:
                    getattr(message, name).extend([b"\xff\x00", b"\xfe"])
                else:
                    setattr(message, name, b"\xff\x00")
            wire = message.SerializeToString()
            self.assertEqual(cls(self.new, message_name).FromString(wire), message)

    def test_legacy_utf8_wire_is_identical_for_all_21_fields(self):
        for message_name, fields in CHANGED.items():
            old = cls(self.old, message_name)()
            for name in fields:
                if old.DESCRIPTOR.fields_by_name[name].is_repeated:
                    getattr(old, name).extend(["test.package", "한글"])
                else:
                    setattr(old, name, "한글 HU")
            wire = old.SerializeToString()
            new = cls(self.new, message_name).FromString(wire)
            self.assertEqual(new.SerializeToString(), wire)
            self.assertEqual(cls(self.old, message_name).FromString(new.SerializeToString()), old)

    def test_native_rust_wire_fixtures_decode(self):
        # Read the exact literals from the Rust tests instead of maintaining copies.
        rust = (ROOT / "tests/sdr_opaque_metadata.rs").read_text(encoding="utf-8")
        blocks = re.findall(r"let raw = \[(.*?)\];", rust, re.S)
        self.assertEqual(len(blocks), 2)
        messages = [cls(self.new).FromString(bytes(int(x, 16) for x in re.findall(r"0x([0-9a-f]+)", block))) for block in blocks]
        self.assertEqual(messages[0].make, b"\xff\x00")
        self.assertEqual(messages[0].headunit_info.model, b"\xfe")
        self.assertEqual(messages[0].services[0].navigation_status_service.minimum_interval_ms, 1000)
        self.assertEqual(messages[1].services[0].bluetooth_service.car_address, b"\xff")
        self.assertEqual(list(messages[1].services[1].vendor_extension_service.package_white_list), [b"\xfd", b"\xfc"])
        self.assertEqual(messages[1].services[2].wifi_projection_service.car_wifi_bssid, b"\xfb")
        for message in messages:
            self.assertTrue(message.IsInitialized())
            message.session_configuration = 1
            self.assertEqual(cls(self.new).FromString(message.SerializeToString()), message)

    def test_unknown_field_survives_edit_and_optional_presence(self):
        raw = b"\x12\x02\xff\x00\xa0\x06\x07"
        message = cls(self.new).FromString(raw)
        self.assertFalse(message.HasField("display_name"))
        message.display_name = b""
        message.session_configuration = 1
        result = message.SerializeToString()
        self.assertIn(b"\xa0\x06\x07", result)
        self.assertTrue(cls(self.new).FromString(result).HasField("display_name"))
        self.assertEqual(cls(self.new).FromString(result).make, b"\xff\x00")

    def test_corrupt_wire_and_required_field_remain_errors(self):
        with self.assertRaises(DecodeError):
            cls(self.new).FromString(b"\x12\x02\xff")
        message = cls(self.new).FromString(b"\x0a\x04\x08\x06\x32\x00")
        self.assertFalse(message.IsInitialized())
        with self.assertRaises(EncodeError):
            message.SerializeToString()


if __name__ == "__main__":
    unittest.main(verbosity=2)

# Upstream issue draft (aa-proxy/buildroot)

**Title:** AAW hostapd template (TKIP + `[HT40+]`) prevents 802.11n/ac on AAWireless 1 CS317 (AP6256)

---

### Device

- AAWireless 1, **CS317** (Rockchip RV1109, Wi-Fi/BT AP6256, bcmdhd)
- Image: `aaw1-cs317-sdcard.img` v1.1.0, with the `aa-proxy-rs` binary rebuilt from v1.1.0 + aa-proxy/aa-proxy-rs@93c8283 (5 GHz detection fix)
- Template: `external/board/aaw-common/rootfs_overlay/etc/hostapd.conf.in` (unchanged on buildroot `main` as of 3f24b6d)

### Problem

With 93c8283 the generated `/var/run/hostapd.conf` correctly gets `hw_mode=a`, `channel=36`, `ieee80211n=1`, `ieee80211ac=1`, but the AP still cannot run 11n/ac because of two fixed lines in the shared AAW template:

```
wpa_pairwise=TKIP
#rsn_pairwise=CCMP
ht_capab=[HT20][HT40+]
```

`hostapd -dd` on CS317:

1. TKIP disables HT/VHT:
   ```
   VHT (IEEE 802.11ac) with WPA/WPA2 requires CCMP/GCMP to be enabled, disabling VHT capabilities
   ```
2. With CCMP enabled, `[HT40+]` is rejected and the AP never comes up:
   ```
   Driver does not support configured HT capability [HT40*]
   wlan0: interface state COUNTRY_UPDATE->DISABLED
   wlan0: AP-DISABLED
   ```

Before 93c8283 the same unit ran 2.4 GHz 11g + TKIP. Android Auto video was about 1.0 MiB/s (~8.4 Mbps) and sessions stalled every 7–10 s (`unexpected transfer stall` / `mitm/MD: transmit failed`) while the phone also kept the HSP link.

### Working change (tested on CS317)

```diff
-wpa_pairwise=TKIP
-#rsn_pairwise=CCMP
+#wpa_pairwise=TKIP
+rsn_pairwise=CCMP
+wmm_enabled=1
 ...
-ht_capab=[HT20][HT40+]
+#ht_capab=[HT20][HT40+]
```

Result: `COUNTRY_UPDATE->ENABLED`, `AP-ENABLED`, client link **802.11ac, channel 36, 433 Mbps**; Android Auto (Galaxy S23 → AAW1 → USB head unit) stable for 10+ minutes in both passthrough and MITM.

`wmm_enabled=1` is required for HT. Without `ht_capab` hostapd uses HT20 defaults; the bcmdhd firmware still negotiated 80 MHz VHT here.

### Notes

- The template is shared by all AAW boards (`aaw-common`). Other radios (AIC8800 / RTL8733) might support HT40, so a board-specific `ht_capab` (or omitting it) may be preferable to removing it everywhere.
- **Upgrade path:** units first booted on v1.1.0 already have `band = "2.4"` and `channel = 6` written to `config.toml`, so the 93c8283 detection fix only helps fresh configs. Upgraded units stay on 2.4 GHz until the user changes `band`/`channel`. A migration or release note may help.
- On CS317 `iw dev wlan0 info` does not report the AP channel with bcmdhd; `AP-ENABLED` in hostapd output was the reliable success signal.

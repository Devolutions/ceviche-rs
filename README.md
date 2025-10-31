# ceviche-rs

[![Cargo](https://img.shields.io/crates/v/ceviche.svg)](https://crates.io/crates/ceviche)
[![Documentation](https://docs.rs/ceviche/badge.svg)](https://docs.rs/ceviche)
![Build](https://github.com/devolutions/ceviche-rs/actions/workflows/build.yaml/badge.svg)

Service/daemon wrapper. Supports Windows, Linux (systemd) and macOS.

## Where we install systemd units and why

### The Challenge

Different Linux distributions place systemd unit files in different locations:
- Debian/Ubuntu and modern RHEL-based systems: `/usr/lib/systemd/system/`
- Older systems and some distributions: `/lib/systemd/system/`
- User-specific overrides: `/etc/systemd/system/`

### Our Approach

This library detects the correct systemd unit directory at runtime using the following strategy:

1. **Environment variable override** (`CEVICHE_SYSTEMD_UNITDIR`): If set, this takes precedence over everything else. Use this when you need explicit control over where units are installed.

2. **pkg-config detection**: We query `pkg-config --variable=systemdsystemunitdir systemd` to get the distribution's preferred location. This works on most modern systems that have systemd development packages installed.

3. **Fallback probing**: If pkg-config is unavailable or doesn't return a result, we probe common directories in order:
   - `/usr/lib/systemd/system`
   - `/lib/systemd/system`

### Caveats and Best Practices

⚠️ **This isn't the ideal approach for packaged software.**

If you're creating distribution packages (`.deb`, `.rpm`, etc.), you should:
- **For Debian/Ubuntu**: Use `dh_installsystemd` or manually install to `${prefix}/lib/systemd/system/`
- **For RPM-based systems**: Use `%{_unitdir}` macro in your spec file
- Let the distribution's packaging tools determine the correct location

This runtime detection is a pragmatic compromise for applications that need to self-register as services without relying on package manager scripts. It works well for:
- Development and testing
- Self-contained applications
- Situations where you can't use distribution-specific packaging

### Usage

By default, ceviche will detect and use the system unit directory. To override:

```bash
# Specify a custom location
export CEVICHE_SYSTEMD_UNITDIR=/etc/systemd/system
./your-application service register
```

### Why Default to System Units?

We default to **system** units (`/usr/lib/systemd/system/` or `/lib/systemd/system/`) rather than user units (`/usr/lib/systemd/user/`, `~/.config/systemd/user/`, etc) because:
- Most services run system-wide
- System units are more common for daemon applications
- You can always override with `CEVICHE_SYSTEMD_UNITDIR` if you need user units

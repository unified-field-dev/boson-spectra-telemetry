# boson-spectra-telemetry examples

| Example | Role |
|---------|------|
| `ops_log_smoke` | `install_ops_log_from_env` + one typed recorder |

## 1. OpsLog — `ops_log_smoke`

```bash
BOSON_TELEMETRY=console CARGO_BUILD_JOBS=1 \
  cargo run -p boson-spectra-telemetry --example ops_log_smoke
```

Success: stdout prints `ops_log_smoke: OK`.

Install Spectra's sink first in real hosts, then this OpsLog install, then build Boson.

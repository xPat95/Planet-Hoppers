"""Native raylib smoke runs and reproducible screenshots; never changes assets."""
from pathlib import Path
import json
import shutil
import subprocess

root = Path(__file__).resolve().parents[1]
out = root / "docs/references/hangar-polish"
out.mkdir(parents=True, exist_ok=True)
exe = root / "target/debug/planet-hoppers.exe"
records = []
raw = root / "target/hangar-polish-runtime"
raw.mkdir(parents=True, exist_ok=True)


def run_logged(args, label):
    with (raw / f"{label}.log").open("wb") as log:
        result = subprocess.run([str(exe), *args], cwd=root, stdout=log, stderr=subprocess.STDOUT, timeout=180)
    text = (raw / f"{label}.log").read_text(encoding="utf-8", errors="replace")
    relevant = [line for line in text.splitlines() if any(marker in line for marker in
                ["_PASS", "DEPARTURE_SAFE", "ENTRY_OUTSIDE", "ERROR", "Error", "WARNING", "Aviso"])]
    (out / f"{label}.log").write_text("\n".join(relevant) + "\n", encoding="utf-8")
    return result, text

for ship in ["halloween", "starwars", "smilingfriends", "minecraft", "regularshow", "southpark"]:
    result, text = run_logged(["--smoke", "--ship", ship], f"launch-{ship}")
    record = dict(ship=ship, exit_code=result.returncode, passed=result.returncode == 0 and "LAUNCH_SMOKE_PASS" in text and "FRONT_END_SMOKE_PASS" in text)
    records.append(record)
    for folder, pattern in [("front-end", "hangar-*.png"), ("solarsystem", f"launch-{ship}.png")]:
        for path in (root / "target" / folder).glob(pattern):
            shutil.copy2(path, out / path.name)
    print(json.dumps(record), flush=True)
    (out / "runtime-summary.json").write_text(json.dumps(records, indent=2), encoding="utf-8")
    if not record["passed"]:
        raise SystemExit(1)

for ship in ["smilingfriends", "starwars", "minecraft"]:
    result, text = run_logged(["--smoke-docking", "--ship", ship], f"docking-{ship}")
    record = dict(mode="docking", ship=ship, exit_code=result.returncode, passed=result.returncode == 0 and "DOCKING_SMOKE_PASS cycles=3" in text and "ENTRY_OUTSIDE" in text)
    records.append(record)
    for path in (root / "target/solarsystem").glob("dock-*.png"):
        shutil.copy2(path, out / f"{ship}-{path.name}")
    print(json.dumps(record), flush=True)
    (out / "runtime-summary.json").write_text(json.dumps(records, indent=2), encoding="utf-8")
    if not record["passed"]:
        raise SystemExit(1)

before = root / "target/hangar-polish-before"
if before.exists():
    (out / "before").mkdir(exist_ok=True)
    for path in before.glob("*.png"):
        shutil.copy2(path, out / "before" / path.name)

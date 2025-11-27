"""Auto-confirm with test validation and diagnostics - Silent Mode"""
import subprocess
import sys
from pathlib import Path
from typing import Dict

PHASE_CRITERIA = {
    "1": {
        "files": ["src-tauri/src/auth.rs", "src/components/WalletConnect.tsx"],
        "tests": ["cargo test --lib auth", "npm test WalletConnect"],
        "health": "curl -s http://localhost:3000/api/health"
    },
    "2": {
        "files": ["src-tauri/src/jupiter.rs", "src/pages/Trading.tsx"],
        "tests": ["cargo test --lib jupiter", "npm test TradingPage"]
    }
}

def silent_check() -> None:
    """Run all checks without prompts, exit on failure."""
    for phase, checks in PHASE_CRITERIA.items():
        if not all(Path(f).exists() for f in checks["files"]):
            sys.exit(f"Phase {phase} missing files")
            
        for test in checks["tests"]:
            result = subprocess.run(test, shell=True, check=False)
            if result.returncode != 0:
                sys.exit(f"Phase {phase} tests failed")

if __name__ == "__main__":
    silent_check()

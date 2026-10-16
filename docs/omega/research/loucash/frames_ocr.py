#!/usr/bin/env python3
"""OCR des frames des 78 videos @0xloucash (texte incruste a l'ecran).

Complement des legendes et des transcriptions : le nom d'une ressource est
souvent AFFICHE a l'ecran sans etre dit ni ecrit dans la legende.

- videos telechargees HORS du depot (/home/tayk0oo/omega-re/loucash-video) ;
- frames extraites par ffmpeg (1 / 6 s, max 10 par video, largeur 720) ;
- OCR tesseract fra+eng ;
- sortie : `ocr.json` dans le corpus (versionnable).

Usage : python3 docs/omega/research/loucash/frames_ocr.py
"""
from __future__ import annotations

import json
import subprocess
import sys
import tempfile
from pathlib import Path

BASE = Path(__file__).resolve().parent
VIDEO = Path("/home/tayk0oo/omega-re/loucash-video")
FRAMES = Path("/home/tayk0oo/omega-re/loucash-frames")
OUT = BASE / "ocr.json"
FPS = "1/6"  # une frame toutes les 6 secondes
MAX_FRAMES = 10


def download(video_id: str) -> Path | None:
    out = VIDEO / f"{video_id}.mp4"
    if out.exists() and out.stat().st_size > 10_000:
        return out
    url = f"https://www.tiktok.com/@0xloucash/video/{video_id}"
    subprocess.run(
        ["yt-dlp", "--no-warnings", "-f", "b[ext=mp4]/b", "-o", str(out), url],
        capture_output=True,
        text=True,
    )
    return out if out.exists() and out.stat().st_size > 10_000 else None


def frames_of(mp4: Path, video_id: str) -> list[Path]:
    d = FRAMES / video_id
    d.mkdir(parents=True, exist_ok=True)
    existing = sorted(d.glob("f*.png"))
    if existing:
        return existing[:MAX_FRAMES]
    subprocess.run(
        [
            "ffmpeg", "-nostdin", "-y", "-i", str(mp4),
            "-vf", f"fps={FPS},scale=720:-1",
            "-frames:v", str(MAX_FRAMES),
            str(d / "f%02d.png"),
        ],
        capture_output=True,
        text=True,
    )
    return sorted(d.glob("f*.png"))[:MAX_FRAMES]


def ocr(path: Path) -> str:
    r = subprocess.run(
        ["tesseract", str(path), "stdout", "-l", "fra+eng", "--psm", "6"],
        capture_output=True,
        text=True,
    )
    return " ".join(r.stdout.split())


def main() -> int:
    vids = json.loads((BASE / "videos.json").read_text())
    VIDEO.mkdir(parents=True, exist_ok=True)
    FRAMES.mkdir(parents=True, exist_ok=True)
    existing: dict[str, str] = json.loads(OUT.read_text()) if OUT.exists() else {}

    print(f"{len(vids)} videos a OCR", flush=True)
    for i, v in enumerate(vids, 1):
        vid = v["id"]
        if existing.get(vid):
            continue
        mp4 = download(vid)
        if mp4 is None:
            print(f"[{i}/{len(vids)}] {vid} : telechargement ECHEC", flush=True)
            continue
        text = " ".join(ocr(f) for f in frames_of(mp4, vid))
        existing[vid] = text
        OUT.write_text(json.dumps(existing, ensure_ascii=False, indent=2))
        print(f"[{i}/{len(vids)}] {vid} : {len(text)} caracteres OCR", flush=True)

    total = sum(len(x) for x in existing.values())
    print(f"TERMINE -> {OUT} ({len(existing)} videos, {total} caracteres OCR)", flush=True)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

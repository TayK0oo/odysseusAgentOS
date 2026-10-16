#!/usr/bin/env python3
"""Transcrit les 78 videos du corpus @0xloucash (Whisper local, GPU si dispo).

Contourne PyAV : decodage par **ffmpeg** vers WAV 16 kHz mono, lecture par le
module standard `wave`, tableaux numpy passes a Whisper (faster-whisper `av`
incompatible avec le PyAV 19 installe).

- l'audio est telecharge HORS du depot (/home/tayk0oo/omega-re/loucash-audio) ;
- sortie utile : `transcripts.json` dans le corpus (versionnable).

Usage : python3 docs/omega/research/loucash/transcribe.py
"""
from __future__ import annotations

import json
import subprocess
import sys
import wave
from pathlib import Path

import numpy as np

BASE = Path(__file__).resolve().parent
AUDIO = Path("/home/tayk0oo/omega-re/loucash-audio")
AUDIO.mkdir(parents=True, exist_ok=True)
OUT = BASE / "transcripts.json"
MODEL = "small"  # bon compromis FR ; 'base' plus rapide, 'medium' plus precis


def download(video_id: str) -> Path | None:
    out = AUDIO / f"{video_id}.mp3"
    if out.exists() and out.stat().st_size > 1024:
        return out
    url = f"https://www.tiktok.com/@0xloucash/video/{video_id}"
    subprocess.run(
        ["yt-dlp", "-x", "--audio-format", "mp3", "--no-warnings", "-o", str(out), url],
        capture_output=True,
        text=True,
    )
    return out if out.exists() and out.stat().st_size > 1024 else None


def to_wav16k(mp3: Path) -> Path | None:
    """Decode en WAV mono 16 kHz via ffmpeg (sans dependre de PyAV)."""
    wav = AUDIO / (mp3.stem + ".16k.wav")
    if wav.exists() and wav.stat().st_size > 1024:
        return wav
    r = subprocess.run(
        ["ffmpeg", "-nostdin", "-y", "-i", str(mp3), "-ac", "1", "-ar", "16000", "-f", "wav", str(wav)],
        capture_output=True,
        text=True,
    )
    return wav if wav.exists() and wav.stat().st_size > 1024 else None


def read_wav_f32(wav: Path) -> np.ndarray:
    with wave.open(str(wav), "rb") as w:
        frames = w.readframes(w.getnframes())
    return np.frombuffer(frames, dtype=np.int16).astype(np.float32) / 32768.0


def main() -> int:
    vids = json.loads((BASE / "videos.json").read_text())
    print(f"{len(vids)} videos a traiter", flush=True)

    wavs: dict[str, Path] = {}
    for i, v in enumerate(vids, 1):
        mp3 = download(v["id"])
        wav = to_wav16k(mp3) if mp3 else None
        if wav:
            wavs[v["id"]] = wav
        if i % 10 == 0 or i == len(vids):
            print(f"[{i}/{len(vids)}] audio pret", flush=True)
    print(f"audio: {len(wavs)}/{len(vids)} prets", flush=True)
    if not wavs:
        print("aucun audio exploitable", file=sys.stderr)
        return 1

    from faster_whisper import WhisperModel

    try:
        model = WhisperModel(MODEL, device="cuda", compute_type="float16")
        print("whisper: cuda float16", flush=True)
    except Exception as e:  # noqa: BLE001
        print(f"whisper: cuda indisponible ({type(e).__name__}) -> cpu int8", flush=True)
        model = WhisperModel(MODEL, device="cpu", compute_type="int8")

    existing = json.loads(OUT.read_text()) if OUT.exists() else {}
    done = 0
    for i, v in enumerate(vids, 1):
        vid = v["id"]
        if existing.get(vid):
            continue
        wav = wavs.get(vid)
        if wav is None:
            continue
        try:
            segments, _info = model.transcribe(
                read_wav_f32(wav), language="fr", vad_filter=True
            )
            text = " ".join(s.text.strip() for s in segments).strip()
        except Exception as e:  # noqa: BLE001
            print(f"[{i}] {vid} ECHEC: {type(e).__name__}: {e}", flush=True)
            continue
        existing[vid] = text
        OUT.write_text(json.dumps(existing, ensure_ascii=False, indent=2))
        done += 1
        print(f"[{i}/{len(vids)}] {vid} : {len(text)} caracteres", flush=True)

    total_chars = sum(len(x) for x in existing.values())
    print(f"TERMINE -> {OUT} ({len(existing)} transcriptions, {total_chars} caracteres)", flush=True)
    print(f"nouvelles cette passe: {done}", flush=True)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

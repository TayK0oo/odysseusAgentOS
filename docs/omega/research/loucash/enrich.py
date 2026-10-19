#!/usr/bin/env python3
"""Enrich @0xloucash TikTok dataset: fetch each video page, extract createTime,
counts (play/digg/comment/share), hashtags, music, and author. Polite rate limit."""
import json, re, time, datetime, urllib.request
from pathlib import Path

BASE = Path(__file__).parent
vids = json.loads((BASE / "videos.json").read_text())
UA = ("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 "
      "(KHTML, like Gecko) Chrome/120.0 Safari/537.36")

def grab(html, *patterns):
    for p in patterns:
        m = re.search(p, html)
        if m:
            return m.group(1)
    return None

for i, v in enumerate(vids):
    vid = v["id"]
    url = f"https://www.tiktok.com/@0xloucash/video/{vid}"
    v["webpage_url"] = url
    try:
        req = urllib.request.Request(url, headers={"User-Agent": UA})
        html = urllib.request.urlopen(req, timeout=25).read().decode("utf-8", "ignore")
        ct = grab(html, r'"createTime":"(\d+)"', r'\\"createTime\\":(\d+)')
        if ct:
            v["create_time"] = int(ct)
            v["upload_date"] = datetime.datetime.utcfromtimestamp(int(ct)).strftime("%Y-%m-%d")
            v["upload_iso"] = datetime.datetime.utcfromtimestamp(int(ct)).isoformat()
        for key, pat in [
            ("play_count", r'"playCount":(\d+)'),
            ("digg_count", r'"diggCount":(\d+)'),
            ("comment_count", r'"commentCount":(\d+)'),
            ("share_count", r'"shareCount":(\d+)'),
            ("collect_count", r'"collectCount":(\d+)'),
        ]:
            val = grab(html, pat)
            if val:
                v[key] = int(val)
        v["hashtags"] = list(dict.fromkeys(re.findall(r'"hashtagName":"([^"]+)"', html)))[:25]
        v["music"] = grab(html, r'"music":{"id":"\d+","title":"([^"]*)"')
        v["author_nickname"] = grab(html, r'"nickname":"([^"]*)"')
        v["enriched"] = True
    except Exception as e:
        v["enrich_error"] = str(e)
        v["enriched"] = False
    print(f"[{i+1}/{len(vids)}] {vid} -> {v.get('upload_date','?')} {v.get('play_count','?')} play", flush=True)
    time.sleep(0.4)

(BASE / "videos_enriched.json").write_text(json.dumps(vids, ensure_ascii=False, indent=2))
print("DONE ->", BASE / "videos_enriched.json")

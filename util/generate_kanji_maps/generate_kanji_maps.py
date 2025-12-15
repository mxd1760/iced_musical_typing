import json
import time
import requests
from pykakasi import kakasi

BASE_URL = "https://kanjiapi.dev/v1/kanji"
GRADES = [1, 2, 3, 4, 5, 6]

# init kakasi (Hepburn-style romaji)
kks = kakasi()
kks.setMode("H", "a")  # Hiragana → ascii
kks.setMode("K", "a")  # Katakana → ascii
kks.setMode("J", "a")  # Kanji (won't really be used here)
kks.setMode("r", "Hepburn")
kks.setMode("s", True)
conv = kks.getConverter()


def kana_to_romaji(kana: str) -> str:
    return conv.do(kana).lower()


def get_grade_kanji(grade: int):
    url = f"{BASE_URL}/grade-{grade}"
    return requests.get(url).json()


def get_kanji_data(kanji: str):
    url = f"{BASE_URL}/{kanji}"
    return requests.get(url).json()


def extract_kanji_kun(reading:str)->str:
    if "." in reading:
        reading = reading.split(".")[0]
    reading = reading.replace("-","").replace(".","").strip()
    return reading

def pick_readings(data, max_readings=3):
    readings = []

    # Primary kun-yomi (first one)
    kun = data.get("kun_readings", [])
    if kun:
        cleaned = extract_kanji_kun(kun[0])
        if cleaned:
          readings.append(kana_to_romaji(cleaned))

    # Primary on-yomi (first one)
    on = data.get("on_readings", [])
    if on:
        readings.append(kana_to_romaji(on[0]))

    # Optional second on-yomi if space allows
    if len(readings) < max_readings and len(on) > 1:
        readings.append(kana_to_romaji(on[1]))

    # Deduplicate while preserving order
    seen = set()
    result = []
    for r in readings:
        if r and r not in seen:
            seen.add(r)
            result.append(r)

    return result[:max_readings]


def build_grade(grade: int):
    print(f"Building Grade {grade}...")
    kanji_list = get_grade_kanji(grade)
    result = {}

    for i, kanji in enumerate(kanji_list):
        data = get_kanji_data(kanji)
        readings = pick_readings(data)
        result[kanji] = readings

        # polite rate limit
        time.sleep(0.12)

        if i % 25 == 0:
            print(f"  {i}/{len(kanji_list)}")

    return result


def main():
    for grade in GRADES:
        grade_map = build_grade(grade)
        filename = f"kanji-grade-{grade}.json"
        with open(filename, "w", encoding="utf-8") as f:
            json.dump(grade_map, f, ensure_ascii=False, indent=2)
        print(f"Wrote {filename}\n")


if __name__ == "__main__":
    main()
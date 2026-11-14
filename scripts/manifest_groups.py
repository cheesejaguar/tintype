"""Manifest reading, grouping columns and name maps shared by the data tooling.

Used by group-manifest.py, audit-manifests.py, sample-authentic-review.py
and build-replay-manifest.py (docs/runbooks/source-family-matching.md).
Standard library only. A script imports it with

    sys.path.insert(0, str(Path(__file__).resolve().parent))
    import manifest_groups

so that it also runs under `python3 -I`.

A manifest is tab-separated `path label generator dataset`, optionally
followed by the grouping columns `parent_id scene_id source_id
generator_family transform` (columns 5 to 9) and anything after them. Paths
are relative to the manifest's directory. This mirrors src/ingest.rs and
src/split.rs: an empty column or `-` means none, a real item's source
defaults to its `real:<name>` key and a generated item's to its dataset,
and the family defaults to the generator name.
"""

import fnmatch
import hashlib
import os
from pathlib import Path

GROUP_COLUMNS = ["parent_id", "scene_id", "source_id", "generator_family", "transform"]


class Row:
    """One manifest row. `path` is as written; `file` is where it points."""

    __slots__ = ("path", "label", "generator", "dataset", "extra", "file", "manifest")

    def __init__(self, path, label, generator, dataset, extra=(), file=None, manifest=None):
        self.path = path
        self.label = int(label)
        self.generator = generator
        self.dataset = dataset
        self.extra = list(extra)
        self.file = file if file is not None else path
        self.manifest = manifest

    def group(self, k):
        """Grouping column `k` of GROUP_COLUMNS, or None when absent, empty or `-`."""
        value = self.extra[k] if k < len(self.extra) else ""
        return value if value and value != "-" else None

    def set_group(self, k, value):
        while len(self.extra) <= k:
            self.extra.append("")
        self.extra[k] = value

    def source(self):
        """`source_id`, or else what src/split.rs assumes (`source_of`)."""
        return self.group(2) or (self.generator if self.label == 0 else self.dataset)

    def family(self):
        """`generator_family`, or else the generator name (`family_of`)."""
        return self.group(3) or self.generator

    def line(self, path=None):
        """The manifest row; trailing empty columns are not written."""
        fields = [self.path if path is None else path, str(self.label), self.generator, self.dataset]
        extra = list(self.extra)
        while extra and not extra[-1]:
            extra.pop()
        return "\t".join(fields + extra)


def read_manifest(path):
    """Rows of a manifest, with `file` resolved against the manifest's directory."""
    path = Path(path)
    root = path.parent
    rows = []
    with path.open(encoding="utf-8", newline="") as stream:
        for number, raw in enumerate(stream, 1):
            line = raw.rstrip("\n").rstrip("\r")
            if not line:
                continue
            f = line.split("\t")
            if len(f) < 4 or f[1] not in ("0", "1"):
                raise ValueError(f"{path}:{number}: not a manifest row: {line[:120]}")
            rows.append(Row(f[0], f[1], f[2], f[3], f[4:], file=str(root / f[0]), manifest=str(path)))
    return rows


def manifest_name(path):
    """A short name: the directory of a `manifest.tsv` (the store), else the file stem."""
    path = Path(path)
    return path.parent.name if path.name == "manifest.tsv" and path.parent.name else path.stem


class Rules:
    """An ordered list of `generator glob, dataset glob, value, basis, note`
    rows; the first row whose two globs match (ignoring case) wins."""

    def __init__(self, rows):
        self.rows = [(g.lower(), d.lower(), value, basis, note) for g, d, value, basis, note in rows]

    @classmethod
    def load(cls, path):
        rows = []
        with open(path, encoding="utf-8") as stream:
            for number, raw in enumerate(stream, 1):
                line = raw.rstrip("\n")
                if not line.strip() or line.startswith("#"):
                    continue
                f = line.split("\t")
                if len(f) < 3 or not f[0] or not f[2]:
                    raise ValueError(f"{path}:{number}: expected `generator glob<TAB>dataset glob<TAB>value[<TAB>basis<TAB>note]`")
                rows.append((f[0], f[1] or "*", f[2], f[3] if len(f) > 3 else "", f[4] if len(f) > 4 else ""))
        return cls(rows)

    def match(self, generator, dataset):
        """`(value, basis)` of the first matching row, or None."""
        g, d = generator.lower(), dataset.lower()
        for gen_glob, data_glob, value, basis, _ in self.rows:
            if fnmatch.fnmatchcase(g, gen_glob) and fnmatch.fnmatchcase(d, data_glob):
                return value, basis
        return None


def family_of(row, rules):
    """`(family, basis)` of a generated item: its column, else the first
    matching rule, else `unknown`. A real item has no family: `(None, None)`."""
    if row.label == 0:
        return None, None
    if row.group(3):
        return row.group(3), "column"
    hit = rules.match(row.generator, row.dataset) if rules else None
    return hit if hit else ("unknown", "unmatched")


def stable_hash(seed, text):
    """A 64-bit integer from a seed and a string, the same on every run and
    platform (Python's own `hash` is salted per process)."""
    digest = hashlib.sha256(f"{seed}\x00{text}".encode("utf-8")).digest()
    return int.from_bytes(digest[:8], "big")


def relative_to(file, directory):
    """`file` as a path relative to `directory`, with forward slashes."""
    return os.path.relpath(file, directory).replace(os.sep, "/")


def is_val(path, val_pct):
    """Whether `tintype train` holds an item out for validation: the same
    path hash as `is_val` in src/data.rs (FNV-1a over the bytes of the item
    path, which is the manifest's directory joined with the row's path, as
    the manifest was named on the command line)."""
    h = 0xCBF29CE484222325
    for b in str(path).encode("utf-8"):
        h = ((h ^ b) * 0x100000001B3) & 0xFFFFFFFFFFFFFFFF
    return (h >> 20) % 100 < val_pct

"""Image transforms shared by the photo benchmark runners."""

import subprocess


def encode_jpeg(source, target, quality, filters=None, *, autorotate=True):
    command = [
        "ffmpeg", "-hide_banner", "-loglevel", "error", "-nostdin",
        "-threads", "1",
    ]
    if not autorotate:
        command.append("-noautorotate")
    command.extend([
        "-i", str(source), "-map_metadata", "-1", "-frames:v", "1",
        "-c:v", "mjpeg", "-threads", "1", "-pix_fmt", "yuvj420p", "-q:v", str(quality),
    ])
    if filters:
        command.extend(["-vf", filters])
    result = subprocess.run(
        [*command, str(target)], stdout=subprocess.DEVNULL, stderr=subprocess.PIPE, text=True,
    )
    if result.returncode or not target.is_file():
        raise ValueError(f"could not encode {source}: {result.stderr.strip()}")


def encode_png(source, target, filters=None, *, autorotate=True):
    """Apply `filters` and store the result losslessly (8-bit RGB PNG), so
    that the filter is the only thing that changed."""
    command = [
        "ffmpeg", "-hide_banner", "-loglevel", "error", "-nostdin",
        "-threads", "1",
    ]
    if not autorotate:
        command.append("-noautorotate")
    command.extend([
        "-i", str(source), "-map_metadata", "-1", "-frames:v", "1",
        "-c:v", "png", "-threads", "1", "-pix_fmt", "rgb24",
    ])
    if filters:
        command.extend(["-vf", filters])
    result = subprocess.run(
        [*command, str(target)], stdout=subprocess.DEVNULL, stderr=subprocess.PIPE, text=True,
    )
    if result.returncode or not target.is_file():
        raise ValueError(f"could not encode {source}: {result.stderr.strip()}")

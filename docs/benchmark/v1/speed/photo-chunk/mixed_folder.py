"""The mixed photo-and-video folder for the parity gate: the 1,000 first files of the speed sample and the 300 smallest
clips of vstore comgenvid, gameplay and pexelsb, interleaved three clips to every ten photos, written to /Volumes/SORAM/mixed. Usage: mixed_folder.py LISTDIR"""
import os, shutil, sys
SP = sys.argv[1]  # holds mixed_videos.txt (size and path under vstore/ of each clip); writes mixed_map.tsv there
vids = ['/Volumes/veracity-data/data/vstore/' + l.split(' ', 1)[1].rstrip('\n') for l in open(SP + '/mixed_videos.txt')]
photos = []
for r, ds, fs in os.walk('/Volumes/SORAM/sample'):
    ds.sort(); photos += sorted(os.path.join(r, f) for f in fs)
photos = photos[:1000]
out = '/Volumes/SORAM/mixed'; os.makedirs(out, exist_ok=True)
vi, pi = iter(vids), iter(photos); m = open(SP + '/mixed_map.tsv', 'w')
for i in range(1300):
    src = next(vi) if i % 13 in (3, 7, 11) else next(pi)
    dst = os.path.join(out, f'm{i:05d}{os.path.splitext(src)[1].lower()}')
    shutil.copyfile(src, dst + '.part'); os.replace(dst + '.part', dst); m.write(f'{os.path.basename(dst)}\t{src}\n')
print('done', len(os.listdir(out)))

# Licence for the trained weights

`veracity.safetensors`, `veracity-video.safetensors` and their `.json` run
specs are released under
**Creative Commons Attribution-NonCommercial-ShareAlike 4.0 International**
(CC BY-NC-SA 4.0, <https://creativecommons.org/licenses/by-nc-sa/4.0/>).

The code in this repository is MIT OR Apache-2.0. The weights carry a stricter
licence because of what they were trained on:

- They were initialised from the Community Forensics ViT-S/16 224 px
  checkpoint (`OwensLab/commfor-model-224`, MIT, Park & Owens, CVPR 2025).
- The training data includes sets released for non-commercial use, the most
  restrictive being Community Forensics (Small), CC BY-NC-SA 4.0. Others:
  OpenFake (CC BY-NC 4.0), So-Fake-Set, GenVideo (CC BY-NC 4.0), AIGVDBench
  (CC BY 4.0), GenBuster-200K (MIT), Defactify, Kinetics-400, Pexels. The
  video checkpoint adds Rapidata's text-to-video preference sets, the
  Human-Animal-Cartoon set and Sunain gameplay (Apache-2.0), Sakugabooru 2025
  (MIT as a collection; the clips are third-party animation) and UCF101.
  [docs/DATA.md](../docs/DATA.md) lists every source.

Whether model weights inherit the licences of their training data is not
settled law. This licence takes the cautious reading. For commercial use,
retrain with `veracity train` on data you have the rights to.

`detect --second-opinion` loads the WaveRep checkpoint. That file is not part
of this repository or of this licence: it is downloaded from its authors
(GRIP-UNINA) and may be used only for informational and non-profit purposes,
with their notices kept and their work cited.

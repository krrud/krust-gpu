# Krust Renderer
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE.md)
## Example Video
<video width="320" height="240" controls>
  <source src="assets/gpu-renderer.webm" type="video/webm">
  Your browser does not support the video tag.
</video>


## Table of Contents
- [Overview](#overview)
- [Usage](#usage)
- [Acknowledgements](#acknowledgements)
- [License](#license)


## Overview <a name="overview"></a>
This project showcases a GPU based raytracter written in Rust for the browser. My focus was on creating a simple, streamlined renderer that is very responsive. Krust uses a highly optimized BVH to make traversing large models a breeze, and leverages MIS to bolster convergance. 

Future improvements currently in development:
- Subdivision (catclark and adaptive)
- Subsurface scattering (diffusion and randomwalk)
- Volumes
- Radiance caching


## Usage <a name="usage"></a>
Feel free to try out a sample scene on the [website](https://krust-gpu.web.app/).
Krust accepts .glb files which can be easily created with most 3D packages.



## Acknowledgements <a name="acknowledgements"></a>
This project was inspired by the work of [Shirley et al.](https://raytracing.github.io/)


## License <a name="license"></a>
This project is licensed under the MIT License - see the LICENSE.md file for details.
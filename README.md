# Krust Renderer
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE.md)
## Example Video
[gpu-renderer.webm](https://github.com/krrud/krust-gpu/assets/1253057/310a4f2c-9713-45c6-8f82-64c0275d366f)


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
Feel free to try out a sample scene on the [website](https://krust-gpu.web.app/). Be sure to enable WebGPU in your browser!
Krust accepts .glb files which can be easily created with most 3D packages.



## Acknowledgements <a name="acknowledgements"></a>
This project was inspired by the work of [Shirley et al.](https://raytracing.github.io/)


## License <a name="license"></a>
This project is licensed under the MIT License - see the LICENSE.md file for details.

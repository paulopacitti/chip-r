# chip-r 🎱
> _"[...] maybe the games don’t have to be more complex, they need to be simple and pleasing. That’s where the sweet spot is!"_ - Allan Alcorn

![](https://raw.githubusercontent.com/paulopacitti/chip-r/main/docs/screenshot.png)

- A simple (with hidden bugs 👀) [CHIP-8](https://en.wikipedia.org/wiki/CHIP-8) emulator built with **rust**.
- Tested on `macos`, but since it uses `sdl2` it can be compiled to multiple targets.

### Usage
 - Compile `core`: `cd core && cargo build`
 - Compile and **run** the emulator: `cd frontend && cargo run <path-to-rom>`

 ### Resources
 - I've built this to learn about emulation development and learn more about `rust`. Here's the guide that helped me through this journey: https://github.com/aquova/chip8-book
 - `CHIP-8` op codes cheatsheet: http://devernay.free.fr/hacks/chip8/C8TECH10.HTM
 - Some `CHIP-8` ROMs are available here: https://www.zophar.net/pdroms/chip8/chip-8-games-pack.html

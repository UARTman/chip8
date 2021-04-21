# CHIP-8 emulator
## Features
- [x] CPU
- [x] Screen
- [x] Debug window
  - [x] Pause/Resume
  - [x] Load ROM
  - [x] Registers
  - [x] Timers
  - [x] Stack
    - [ ] Stack manipulation
  - [x] Loading a ROM from file
  - [ ] Keyboard
    - [ ] Key manipulation
    - [ ] Key binding manipulation
  - [ ] Step-by-step execution
  - [ ] View RAM
  - [ ] View selected sprites 
- [ ] Keyboard
- [ ] Sound
## Quirks
### Running on Windows
The emulator requires you have MSVC++ 2015 libraries.
### Binary size
Linux debug binaries are very big (200 MB), but release ones are 8MB (which can be easily stripped to 5MB)
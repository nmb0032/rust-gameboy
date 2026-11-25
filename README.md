# Rust Game Boy

A learning-focused emulator for the original Nintendo Game Boy (DMG), written in Rust. The goal is to model the hardware accurately enough to run real ROMs while keeping each subsystem independently testable and understandable.

## Project Goals

- Implement the DMG CPU, memory map, timers, display pipeline, audio, interrupts, and joypad.
- Pass widely used Game Boy CPU, timing, and graphics test ROMs.
- Provide a reusable emulator core with separate display, input, and audio frontends.
- Prioritize correctness, deterministic behavior, and clear hardware-oriented code over shortcuts.
- Document the architecture and important hardware behavior as the project grows.

## Scope

The first release targets the original monochrome Game Boy. Planned support is intentionally limited to:

- ROM-only and MBC1 cartridges initially, followed by MBC3 and MBC5.
- Debuggable CPU and memory execution from a command-line frontend.
- A desktop frontend with keyboard input, frame rendering, audio, and save support.
- Step-based unit and integration tests for hardware behavior.

Game Boy Color, Super Game Boy features, obscure peripherals, and network features are future work.

## Roadmap

Each phase ends with something you can *run* and *see working*. Phases build on each other, so finish (or at least get passing) one before starting the next. Every phase has the same shape:

- **Goal** — what you'll have when you're done.
- **What you'll learn** — the hardware and Rust concepts involved.
- **Steps** — small, checkable chunks. Each should be a sitting or two of work.
- **Check your understanding** — questions to answer before moving on.
- **Common pitfalls** — mistakes almost every emulator author makes.

Keep [Pan Docs](https://gbdev.io/pandocs/) open the entire time; it is the primary reference for every phase.

### Phase 0: Project Foundation

**Goal:** Get a working Rust workspace where the frontend can create and drive an (empty) emulator. No Game Boy hardware yet — this phase is about building the skeleton everything else hangs on.

**What you'll learn**

- How Cargo workspaces let multiple crates share one build and lockfile.
- How Rust's module privacy (`mod`, `pub`, `pub use`) controls what one crate can see of another.
- Why the emulator core should know nothing about windows, audio devices, or keyboards.
- How unit tests and integration tests differ in Rust.

#### Step 1: Set up the workspace

- [x] Initialize Git and add a `.gitignore`.
- [x] Create a root `Cargo.toml` workspace with a `gameboy-core` library and a `gameboy-frontend` binary.
- [x] Give the crates descriptive names.

> Why: splitting the core from the frontend from day one forces a clean boundary. Later you could add a web or terminal frontend without touching hardware code.

#### Step 2: Connect the crates

- [x] Add `gameboy-core` as a path dependency in `gameboy-frontend/Cargo.toml`.
- [x] Replace the generated `add` function in `gameboy-core` with an `Emulator` struct.
- [x] Call `Emulator::new()` from the frontend's `main` and print something to prove it works.

> Try it: run `cargo run -p gameboy-frontend` from the repository root.

#### Step 3: Decide what the core exposes

- [x] Keep internal modules private (`mod cpu;`, not `pub mod cpu;`).
- [x] Re-export only what a frontend needs from `lib.rs` (for now, just `Emulator`).
- [x] Add a `step()` method that does nothing yet, so the frontend has a loop to call.

> Why: the frontend should only ever *ask* the emulator to run and *read* its output. If it can reach into the CPU directly, refactors become painful. A good test: could you rewrite the CPU entirely without changing frontend code?

#### Step 4: Write your first tests

- [x] Add a unit test inside `gameboy-core` (a `#[cfg(test)] mod tests` block) that creates an `Emulator`.
- [x] Add an integration test in `crates/gameboy-core/tests/` that uses only the public API.

> Why: unit tests can see private items; integration tests see exactly what a frontend sees. Having both early makes the public boundary concrete.

#### Step 5: Build good habits

- [x] Run `cargo fmt` and `cargo clippy` and fix anything they report.
- [x] Commit the working skeleton.

**Check your understanding before moving on**

- Can you explain why `gameboy-core` has no dependencies on graphics or audio libraries?
- If you made `cpu` a `pub mod`, what could the frontend do that it shouldn't?
- What does `cargo test` run from the workspace root, and how is that different from running it inside one crate?

**Common pitfalls**

- Creating lots of empty modules "for later". Add modules when the phase that needs them begins.
- Designing the full framebuffer/audio/input API now. You don't know enough yet; it will become clear in Phases 5, 6, and 8.
- Committing ROM files. Keep them local; the `.gitignore` already excludes `*.gb`.

**Phase 0 is done when** `cargo build`, `cargo test`, and `cargo run -p gameboy-frontend` all succeed from the repository root, and the frontend creates an `Emulator` through the core's public API.

*Optional extras:* add a GitHub Actions workflow running `fmt`, `clippy`, and `test`; add a `rustfmt.toml` if you want non-default formatting.

### Phase 1: Memory Bus and Cartridge Loading

**Goal:** Load a real ROM file and read bytes from it through a memory bus that mirrors the Game Boy's address space.

**What you'll learn**

- The Game Boy's 16-bit memory map and why every component (CPU, PPU, timer, joypad) talks through it.
- How a cartridge header describes the game and its hardware.
- Rust ownership patterns for shared state: one `Bus` that owns the components instead of components holding references to each other.
- Loading files in the frontend and passing bytes to the core, so the core never touches the filesystem.

#### Step 1: Read the memory map

- [x] Read the Pan Docs *Memory Map* page and write the regions as constants or a comment table in your code.

| Range | Contents |
| --- | --- |
| `0000-3FFF` | ROM bank 0 |
| `4000-7FFF` | ROM bank 1+ (switchable later) |
| `8000-9FFF` | Video RAM |
| `A000-BFFF` | Cartridge RAM |
| `C000-DFFF` | Work RAM |
| `E000-FDFF` | Echo of `C000-DDFF` |
| `FE00-FE9F` | OAM (sprite table) |
| `FEA0-FEFF` | Unusable |
| `FF00-FF7F` | I/O registers |
| `FF80-FFFE` | High RAM |
| `FFFF` | Interrupt enable register |

#### Step 2: Build the bus

- [x] Create a `Bus` with `read(addr: u16) -> u8` and `write(addr: u16, value: u8)`.
- [x] Back work RAM, VRAM, OAM, and high RAM with fixed-size arrays.
- [x] Store unhandled I/O registers in a plain array for now; you'll route them to real components later.
- [x] Unit test each region: write a value, read it back. Test the echo region mirrors work RAM.

> Why: once every read and write goes through one function, adding a new component later is just another `match` arm.

#### Step 3: Load a cartridge

- [x] In the frontend, read a `.gb` file from a command-line argument and pass the bytes to `Emulator::new`.
- [x] Parse the header (`0x0100-0x014F`): title (`0x0134`), cartridge type (`0x0147`), ROM size (`0x0148`), RAM size (`0x0149`).
- [x] Verify the header checksum at `0x014D` and return an error if it fails.
- [x] Support only ROM-only cartridges (type `0x00`) for now; return a clear error for other types.

> Try it: print the parsed title and cartridge type for a homebrew ROM.

**Check your understanding**

- Why can a 16-bit address only reach 64 KiB, and how do games bigger than 32 KiB work?
- What should happen when the CPU writes to ROM? (Hint: on real cartridges it's not ignored — it controls bank switching, which you'll add in Phase 7.)

**Common pitfalls**

- Off-by-one errors on region boundaries. Test the first and last address of every region.
- Giving components references to each other. Have the `Bus` own them; you'll fight the borrow checker much less.

**Phase 1 is done when** the emulator loads a ROM-only cartridge, prints its header, and reads correct bytes through the bus.

### Phase 2: The CPU

**Goal:** Execute real Game Boy machine code one instruction at a time.

**What you'll learn**

- The Sharp SM83 CPU (often called LR35902): registers, flags, and the fetch → decode → execute loop.
- How opcodes are laid out in a table, and how to spot patterns so you don't write 500 hand-written cases.
- Bit manipulation in Rust: masks, shifts, `wrapping_add`, `overflowing_sub`, and carry detection.
- What a machine cycle (M-cycle, 4 clock ticks) is and why instructions report how long they took.

#### Step 1: Registers

- [ ] Model `A, F, B, C, D, E, H, L`, `SP`, and `PC`.
- [ ] Add helpers to read/write the 16-bit pairs `AF`, `BC`, `DE`, `HL`.
- [ ] Model the flags in `F`: Z (bit 7), N (bit 6), H (bit 5), C (bit 4). The low 4 bits of `F` are always zero.
- [ ] Initialize registers to the DMG post-boot values (`AF=01B0`, `BC=0013`, `DE=00D8`, `HL=014D`, `SP=FFFE`, `PC=0100`) so you can skip the boot ROM.

#### Step 2: The execution loop

- [ ] Implement `step()`: fetch the byte at `PC`, advance `PC`, decode, execute, and return the cycles used.
- [ ] Panic with the opcode and address on unimplemented instructions; this tells you exactly what to build next.
- [ ] Implement `NOP`, `JP nn`, and `LD r, n` and run them against a ROM.

> Why: panicking on unknown opcodes turns the ROM itself into your to-do list.

#### Step 3: Instruction groups

Implement in this order, unit testing each group before moving on:

- [ ] 8-bit loads (`LD r, r'`, `LD r, (HL)`, `LD (HL), r`, `LDH`, etc.).
- [ ] 16-bit loads and stack operations (`LD rr, nn`, `PUSH`, `POP`).
- [ ] 8-bit arithmetic and logic (`ADD`, `ADC`, `SUB`, `SBC`, `AND`, `XOR`, `OR`, `CP`, `INC`, `DEC`).
- [ ] 16-bit arithmetic (`ADD HL, rr`, `INC rr`, `DEC rr`, `ADD SP, e`).
- [ ] Jumps, calls, and returns, including conditional versions (`JR`, `JP`, `CALL`, `RET`, `RST`).
- [ ] Rotates, `DAA`, `CPL`, `SCF`, `CCF`.
- [ ] The `CB`-prefixed table: rotates, shifts, `SWAP`, `BIT`, `RES`, `SET`.
- [ ] `DI`, `EI`, and `HALT` as stubs (finished in Phase 4).

> Tip: opcodes encode their register operands in bit fields. For example, `LD r, r'` is `01 ddd sss`. Decoding those bits lets one function handle 63 opcodes.

#### Step 4: Cycle counts

- [ ] Return the correct cycle count from every instruction, including the extra cycles for taken conditional branches.

**Check your understanding**

- When does `ADD` set the half-carry flag, and why does `DAA` need it?
- Why is `PUSH AF` / `POP AF` special with respect to the low bits of `F`?

**Common pitfalls**

- Wrong half-carry calculations. These are the most common CPU bug; test them explicitly.
- Forgetting that 16-bit values are stored little-endian (low byte first).
- Implementing all 512 opcodes before running anything. Get a few working end to end first.

**Phase 2 is done when** every opcode is implemented and has at least a basic unit test.

### Phase 3: Validating the CPU

**Goal:** Prove the CPU is correct using the community test ROMs before building anything on top of it.

**What you'll learn**

- How test ROMs report results, and how to read them without a screen.
- How to debug by comparing execution logs against a known-good emulator.
- Building simple developer tooling: a trace log and a step debugger.

#### Step 1: Serial output

- [ ] Capture writes to the serial registers: when `0x81` is written to `SC` (`FF02`), record the byte in `SB` (`FF01`) as a character.
- [ ] Print captured characters from the frontend.

> Why: Blargg's test ROMs print their results over the serial port, so you can see "Passed" or "Failed" before you have any graphics.

#### Step 2: Run Blargg's `cpu_instrs`

- [ ] Run each of the 11 individual `cpu_instrs` test ROMs and fix failures one at a time.
- [ ] Turn passing test ROMs into integration tests that check the serial output.

#### Step 3: Debugging tools

- [ ] Add an optional trace log that prints registers and `PC` before each instruction.
- [ ] Compare your log against a reference log (for example, with the *Gameboy Doctor* tool) to find the first instruction that diverges.
- [ ] Optionally run the SM83 single-step JSON tests, which check every opcode against expected register and memory state.

> Try it: when a test fails, the first divergent line in the log almost always points at the bug.

**Check your understanding**

- Why is it worth spending a whole phase on CPU correctness before graphics?

**Common pitfalls**

- Moving on with some `cpu_instrs` tests failing. Bugs here surface later as baffling graphics glitches.

**Phase 3 is done when** all 11 `cpu_instrs` tests pass.

### Phase 4: Interrupts and Timers

**Goal:** Let hardware interrupt the CPU, and give games a sense of time.

**What you'll learn**

- How interrupts work: request flags (`IF`, `FF0F`), enable flags (`IE`, `FFFF`), and the master enable (`IME`).
- Interrupt priority and the fixed jump vectors.
- How the timer counts clock ticks and why it's built on a hidden 16-bit counter.
- How to keep multiple components in sync by advancing them by the same number of cycles.

#### Step 1: Interrupts

- [ ] Model `IF`, `IE`, and `IME`.
- [ ] After each instruction, if an enabled interrupt is requested and `IME` is set, push `PC`, jump to its vector, and clear `IME` and the request bit.
- [ ] Use the correct priority and vectors: VBlank `0x40`, LCD STAT `0x48`, Timer `0x50`, Serial `0x58`, Joypad `0x60`.
- [ ] Make `EI` take effect after the *following* instruction; finish `RETI` and `HALT`.

#### Step 2: Timers

- [ ] Implement `DIV` (`FF04`) as the upper 8 bits of an internal 16-bit counter; any write resets it to zero.
- [ ] Implement `TIMA` (`FF05`), `TMA` (`FF06`), and `TAC` (`FF07`).
- [ ] When `TIMA` overflows, reload it from `TMA` and request a timer interrupt.

#### Step 3: Keep components in sync

- [ ] After each CPU step, advance the timer by the same number of cycles.

> Why: this "run the CPU, then catch everything else up" loop is the heart of the emulator. Every later component plugs into it.

**Check your understanding**

- What's the difference between an interrupt being *requested*, *enabled*, and *serviced*?
- Why does `HALT` need interrupts to be useful?

**Common pitfalls**

- Servicing interrupts when `IME` is off. `HALT` should still wake up, but no jump should occur.
- Forgetting the delay on `EI`.

**Phase 4 is done when** Blargg's `instr_timing` test passes and the `cpu_instrs` interrupt test still passes.

### Phase 5: Graphics — Background and Window

**Goal:** See pixels on screen. Render the background and window layers in a desktop window.

**What you'll learn**

- How the PPU draws 154 scanlines per frame (144 visible + 10 VBlank), 456 clock ticks each.
- PPU modes: OAM scan (2), drawing (3), HBlank (0), and VBlank (1).
- Tiles: 8×8 pixel images stored in 16 bytes using 2 bits per pixel.
- Tile maps, scrolling (`SCX`, `SCY`), palettes (`BGP`), and the window (`WX`, `WY`).
- Presenting a framebuffer with a Rust graphics crate (for example `minifb` or `pixels`).

#### Step 1: PPU timing

- [ ] Track the current scanline (`LY`, `FF44`) and mode, advancing with CPU cycles.
- [ ] Request a VBlank interrupt when `LY` reaches 144.
- [ ] Implement `STAT` (`FF41`) and the `LY == LYC` comparison with its interrupt.

#### Step 2: Draw tiles

- [ ] Decode a tile from VRAM into palette indices 0–3.
- [ ] Write a debug viewer that draws every tile in VRAM. Seeing recognizable tiles proves decoding works.

#### Step 3: Background and window

- [ ] Render the background line by line using `LCDC` (`FF40`) to choose tile maps and tile data addressing.
- [ ] Apply `SCX`/`SCY` scrolling with wraparound.
- [ ] Render the window layer using `WX` (offset by 7) and `WY`.
- [ ] Map colors through `BGP`.

#### Step 4: Show it

- [ ] Expose a 160×144 framebuffer from the core.
- [ ] Add `run_frame()` to the emulator so the frontend can run one frame at a time.
- [ ] Display frames in a window at roughly 60 frames per second.

> Try it: the Nintendo logo scrolling down is the first sign everything is wired up — if you run the boot ROM. Otherwise, a homebrew or test ROM's title screen will appear.

**Check your understanding**

- Why does tile data have two addressing modes (`8000` unsigned and `8800` signed)?
- What draws first on each line: the background or the window?

**Common pitfalls**

- Rendering a whole frame at once instead of line by line. Games change scroll registers mid-frame for effects.
- Mixing up the order of the two bytes that form each tile row.

**Phase 5 is done when** a ROM's background and window render correctly in a window on screen.

### Phase 6: Sprites, Input, and Your First Game

**Goal:** Play a real game. *Tetris* is a ROM-only cartridge, so it's the classic first target.

**What you'll learn**

- Sprites (objects): the OAM table, 8×8 and 8×16 sizes, flipping, palettes, and priority.
- How OAM DMA quickly copies sprite data into OAM.
- How the joypad register works with active-low bits and selection lines.
- Frame pacing and passing input from the frontend to the core.

#### Step 1: Sprites

- [ ] Parse the 40 OAM entries (4 bytes each: Y, X, tile, attributes). Remember the Y offset of 16 and X offset of 8.
- [ ] Select up to 10 sprites per scanline.
- [ ] Draw sprites with `OBP0`/`OBP1`, horizontal/vertical flip, and background priority.
- [ ] Support 8×16 sprites.

#### Step 2: OAM DMA

- [ ] Writing to `DMA` (`FF46`) copies 160 bytes from `XX00` into OAM.

#### Step 3: Joypad

- [ ] Implement `P1` (`FF00`): games select directions or buttons via bits 4–5 and read pressed keys as `0`.
- [ ] Request a joypad interrupt on a key press.
- [ ] Add a frontend-independent input type in the core, and map the keyboard to it in the frontend.

> Try it: boot *Tetris* (your own legally owned copy) and play a round.

**Check your understanding**

- Why do sprites need a priority rule, and what decides which sprite is on top?
- Why must the core's input type not mention keyboard keys?

**Common pitfalls**

- Treating joypad bits as active-high. A pressed button reads as `0`.
- Ignoring the 10-sprites-per-line limit; some games rely on it for flicker effects.

**Phase 6 is done when** you can play a ROM-only game with keyboard input and correct sprites. The `dmg-acid2` test ROM is an excellent check of sprite and background rendering.

### Phase 7: Memory Bank Controllers and Saves

**Goal:** Run most of the Game Boy library by supporting larger cartridges and battery-backed saves.

**What you'll learn**

- Bank switching: how writes to the ROM area select which part of a large ROM appears in `4000-7FFF`.
- Rust traits as a way to swap cartridge implementations behind one interface.
- Persisting save data (`.sav` files) in the frontend.

#### Step 1: A cartridge trait

- [ ] Define a trait for cartridge reads and writes, and move ROM-only support behind it.

#### Step 2: MBC1

- [ ] Implement ROM bank switching, RAM enable, and RAM bank switching.
- [ ] Handle the quirk where selecting bank 0 selects bank 1.

#### Step 3: Saves

- [ ] Expose cartridge RAM from the core, and load/save it as a `.sav` file in the frontend for battery-backed cartridges.

#### Step 4: More controllers

- [ ] Add MBC3 (used by *Pokémon Red/Blue*), including the real-time clock if you want.
- [ ] Add MBC5.

**Check your understanding**

- Why can't the core write `.sav` files itself?

**Common pitfalls**

- Masking bank numbers incorrectly for ROMs smaller than the controller's maximum.

**Phase 7 is done when** an MBC1 game and an MBC3 game play and their saves survive a restart.

### Phase 8: Audio

**Goal:** Hear the music and sound effects.

**What you'll learn**

- The four sound channels: two square waves (one with frequency sweep), a programmable wave channel, and a noise channel.
- The frame sequencer that clocks length counters, volume envelopes, and sweep at 512 Hz.
- Sample rates, buffering, and keeping emulation in sync with an audio device (for example with `cpal`).

#### Step 1: One channel first

- [ ] Implement channel 2 (square wave, no sweep) and its registers.
- [ ] Generate samples, and play them through the frontend.

> Why: hearing a single correct tone proves the whole audio pipeline works before adding complexity.

#### Step 2: The frame sequencer

- [ ] Clock length counters, envelopes, and sweep from the frame sequencer.

#### Step 3: The remaining channels

- [ ] Channel 1 (square wave with sweep).
- [ ] Channel 3 (wave channel using wave RAM `FF30-FF3F`).
- [ ] Channel 4 (noise using a linear-feedback shift register).

#### Step 4: Mixing and sync

- [ ] Mix channels using the panning (`NR51`) and master volume (`NR50`) registers.
- [ ] Use the audio buffer to keep emulation at the correct speed without crackling.

**Check your understanding**

- Why can't the emulator just produce one sample per CPU cycle?

**Common pitfalls**

- Buffer underruns causing crackles. A slightly larger buffer is usually the fix.
- Forgetting that disabling sound via `NR52` clears the sound registers.

**Phase 8 is done when** games play recognizable music and sound effects without crackling.

### Phase 9: Accuracy, Polish, and Compatibility

**Goal:** Turn a working emulator into a complete one: accurate, comfortable to use, and broadly compatible.

**What you'll learn**

- Hardware edge cases that games rely on (timing quirks, mid-scanline effects).
- Serialization for savestates.
- Profiling Rust code once correctness is established.

#### Step 1: Test suites

- [ ] Run Blargg's `mem_timing` and `dmg_sound` tests.
- [ ] Run a selection of Mooneye test ROMs, and record your results in this README.

#### Step 2: Accuracy improvements (pick based on what fails)

- [ ] Move from instruction-level to M-cycle-level timing if tests or games need it.
- [ ] Restrict CPU access to VRAM and OAM during the PPU modes that lock them.
- [ ] Implement the `HALT` bug and other documented quirks.

#### Step 3: Usability

- [ ] Pause, reset, fast-forward, and screenshot controls.
- [ ] Savestates.
- [ ] Optional: a debugger view showing registers, memory, and tiles.
- [ ] Optional: run the original boot ROM if you provide your own copy.

#### Step 4: Compatibility

- [ ] Keep a list of games you've tested and how well they work.

**Phase 9 is done when** you can pick a game from your library and expect it to just work.

## Architecture Direction

The implementation should separate hardware state from platform concerns:

- `core`: CPU, bus, cartridges, PPU, APU, timers, joypad, interrupts, and emulator coordination.
- `frontend`: windowing, rendering, input, audio, ROM selection, and settings.
- `tests`: focused fixtures, regression ROM metadata, and integration utilities.

The core should not depend on a GUI or operating-system APIs. It should advance in cycles or fixed steps and expose a framebuffer/audio stream plus explicit input commands. This keeps tests deterministic and allows future frontends without rewriting hardware code.

### Planned Folder Layout

```text
rust-gameboy/
├── Cargo.toml
├── Cargo.lock
├── README.md
├── rustfmt.toml
├── .gitignore
├── crates/
│   ├── core/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── emulator.rs
│   │       ├── bus.rs
│   │       ├── interrupts.rs
│   │       ├── timer.rs
│   │       ├── joypad.rs
│   │       ├── cpu/
│   │       │   ├── mod.rs
│   │       │   ├── registers.rs
│   │       │   ├── instructions.rs
│   │       │   └── decode.rs
│   │       ├── cartridge/
│   │       │   ├── mod.rs
│   │       │   ├── header.rs
│   │       │   ├── rom_only.rs
│   │       │   └── mbc1.rs
│   │       ├── ppu/
│   │       │   ├── mod.rs
│   │       │   ├── background.rs
│   │       │   └── sprites.rs
│   │       └── apu/
│   │           ├── mod.rs
│   │           ├── pulse.rs
│   │           ├── wave.rs
│   │           └── noise.rs
│   └── frontend/
│       ├── Cargo.toml
│       └── src/
│           ├── main.rs
│           ├── app.rs
│           ├── audio.rs
│           ├── input.rs
│           └── persistence.rs
├── tests/
│   ├── integration/
│   ├── fixtures/
│   └── roms/                 # Local test ROMs; ignored by Git
├── benches/
└── .github/
    └── workflows/
```

The tree above is a target structure rather than a requirement to create empty modules early. Hardware modules should be added as their roadmap phase begins, keeping the initial workspace small and each subsystem independently testable.

## Development Workflow

Recommended progression is one tested hardware boundary at a time:

1. Add a small failing test for a register, opcode, timing edge, or memory rule.
2. Implement the minimum hardware behavior needed by that test.
3. Run focused tests before the full suite.
4. Run formatter and linter checks.
5. Record compatibility results when behavior is exercised by a test ROM.

ROMs are not committed to the repository unless their licensing clearly permits redistribution. The test configuration will point to locally supplied ROMs.

## Validation Strategy

- Unit tests for CPU decoding, opcode execution, memory mapping, cartridge banking, timer edges, and PPU fetch state.
- Integration tests that run complete instruction or graphics test ROM sequences to a known result.
- Snapshot tests for carefully chosen CPU and display states.
- Compatibility tracking that distinguishes unsupported ROM behavior from incorrect emulator behavior.
- Benchmarks for long-running frame stepping once correctness is established.

## Planned Commands

Once the Rust workspace is scaffolded, the expected developer commands will be:

```bash
cargo build
cargo test
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo run -p frontend -- path/to/game.gb
```

## References and Research

Development should use hardware references and non-commercial test resources, including the Pan Docs, relevant opcode references, and publicly available emulator test ROMs. Any third-party material included in the repository must be reviewed for licensing and attribution requirements.

## License

The project license will be selected when the initial codebase is created. ROM files and copyrighted firmware are not part of this repository.

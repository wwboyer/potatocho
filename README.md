# PotatOcho

### What is this?

PotatOcho is a my work-in-progress Chip-8 emulator coded in Rust. I decided to make this as a way to both learn Rust while also dusting off some of the coding cobwebs. In other words, this is allowing me to simultaneously Rust *and* de-rust (ha ha, puns). Speaking of puns, I called it PotatOcho because the "Chip" in Chip-8 reminded me of potato chips, and "ocho" is Spanish for 8, so you put them together and you get "PotatOcho". I know, I know, I'm a comedic genius. I'll be here all week.

### How far along is it?

PotatOcho utilizes SDL to render at an internal resolution of 64x32 to a window with a resolution of 1280x640, as well as to accept keypad input.

For testing compatibility, I'm using [Timendus's amazing *Chip8 Test Suite*](https://github.com/Timendus/chip8-test-suite). Seriously, without this, testing would be so much more annoying than it already is.

### Current Test Results:

#### Test 1: IBM Logo

![Image displaying the IBM logo.](https://i.imgur.com/HNssGy4.png)

#### Test 2: Corax89's Opcode Test

![Image displaying the results of the Corax89 Opcode Test. All tested opcodes pass.](https://i.imgur.com/GQZlV28.png)

#### Test 3: Flags

![Image displaying the results of the Flags Test. All tested flags are OK.](https://i.imgur.com/gacmO9W.png)

#### Test 4: Quirks (Chip-8 Test Only)

![Image displaying the results of the Quirks test.](https://i.imgur.com/vntv2Wc.png)

#### Test 5: Keypad

![Image displaying the results of the Keypad test.](https://i.imgur.com/cPjAhfN.png)

### Credits

* [Cowgod's Chip-8 Technical Reference](http://devernay.free.fr/hacks/chip8/C8TECH10.HTM) for being an incredibly in-depth and well-documented resource on how the Chip-8 interpreter functions.

* [Timendus's Chip-8 Test Suite](https://github.com/Timendus/chip8-test-suite) for being an equally amazing testing suite that helped me catch many an issue (like me messing up a `match` statement and not actually clearing the screen when 00E0 happens. Oops.)

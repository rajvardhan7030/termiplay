# 🎬 TermiPlay

**TermiPlay** is a fun and simple video player that runs inside your command-line interface (your terminal!). It allows you to watch videos directly in your coding environment using various artistic and high-performance styles.

---

## 🛠️ Getting Started (The Setup)

Before you can play videos, you need to set up two main tools on your computer. Don't worry, we'll walk you through it!

### 1. Install Rust (The Engine)
TermiPlay is built with a language called Rust. To run it, you need the Rust toolset.
*   **How to install:** Go to [rustup.rs](https://rustup.rs/) and follow the simple instructions for your operating system (Windows, macOS, or Linux). 
*   **Verify:** Open your terminal and type `cargo --version`. If you see a version number, you're good to go!

### 2. Install FFmpeg development libraries (The Video Decoder)
TermiPlay links against FFmpeg through Rust bindings, so you need the FFmpeg command-line tool and the development headers/libraries.
*   **Windows:** Download from [gyan.dev](https://www.gyan.dev/ffmpeg/builds/), extract it, and add the `bin` folder to your "Environment Variables" (PATH).
*   **macOS:** Open your terminal and type `brew install ffmpeg` (requires [Homebrew](https://brew.sh/)).
*   **Linux (Ubuntu/Debian):** Open your terminal and type:
    ```bash
    sudo apt update
    sudo apt install ffmpeg pkg-config clang libclang-dev libavutil-dev libavcodec-dev libavformat-dev libswscale-dev libswresample-dev
    ```

---

## 📥 Downloading TermiPlay

Once the tools above are installed, you can get the code:

1.  Open your terminal.
2.  Type the following command to download the project:
    ```bash
    git clone https://github.com/Rajvardhan7030/termiplay.git
    ```
3.  Enter the project folder:
    ```bash
    cd termiplay
    ```

---

## 🚀 How to Play a Video

To play a video, use the following command:

```bash
cargo run -- <path_to_your_video_file>
```

**Example:**
```bash
cargo run -- my_movie.mp4
```

### 🎨 Rendering Modes (Art Styles)
You can change how the video looks by adding the `--mode` option:

*   **`--mode=unicode` (Default):** High-resolution rendering using half-block characters (▀). Best balance of quality and performance.
*   **`--mode=ansi`:** Classic colored block style using full blocks (█).
*   **`--mode=ascii`:** Retro grayscale style using standard ASCII characters.
*   **`--mode=kitty`:** Native terminal graphics for ultra-high resolution. Requires the [Kitty Terminal](https://sw.kovidgoyal.net/kitty/) or another terminal that exposes Kitty graphics support through Kitty-compatible environment variables.

**Example with high resolution:**
```bash
cargo run -- my_movie.mp4 --mode=kitty
```

---

## ⚡ Performance Tips

If the video feels slow or "blinks" in Kitty mode, use the "Low Power" mode:

```bash
cargo run -- my_movie.mp4 --mode=kitty --low
```
*This reduces the internal rendering resolution, making it much smoother on older hardware.*

---

## 🛠️ Technical Details

TermiPlay is built for performance and efficiency:
*   **Language:** Written in 100% Rust for memory safety and speed.
*   **Decoding:** Powered by `ffmpeg-next` (FFmpeg bindings) for broad format support.
*   **Concurrency:** Uses standard Rust threads and `crossbeam-channel` for a multi-threaded decoding and rendering pipeline.
*   **Audio:** Real-time audio playback via `rodio`.

---

## 🎮 Keyboard Controls

*   **Q or Esc:** Close the player and return to your terminal.
*   *(More controls like Pause/Seek are coming soon!)*

---

## 🔍 Troubleshooting (If things go wrong)

**"I see a black screen in Kitty mode!"**
> **Fix:** Make sure you are running inside Kitty or a Kitty-compatible terminal. Otherwise, use `--mode=unicode`.

**"`cargo run` fails with `Package 'libavutil' not found`."**
> **Fix:** Install the FFmpeg development packages listed in the setup section. On Linux, the plain `ffmpeg` package is not enough to compile the Rust FFmpeg bindings.

**"The video is laggy or flickering."**
> **Fix:** Use the `--low` flag (if in Kitty mode) or try a simpler mode like `--mode=ascii`.

**"Error: Could not find video stream."**
> **Fix:** Make sure the path to your video file is correct and that the file is not corrupted.

---

## 🤝 Credits & License
TermiPlay is built by the TermiPlay Team and is licensed under the MIT License. Enjoy your terminal cinema! 🍿

# 🎬 TermiPlay

**TermiPlay** is a fun and simple video player that runs inside your command-line interface (your terminal!). It allows you to watch videos directly in your coding environment using various artistic and high-performance styles.

---

## 🛠️ Getting Started (The Setup)

Before you can play videos, you need to set up two main tools on your computer. Don't worry, we'll walk you through it!

### 1. Install Rust (The Engine)
TermiPlay is built with a language called Rust. To run it, you need the Rust toolset.
*   **How to install:** Go to [rustup.rs](https://rustup.rs/) and follow the simple instructions for your operating system (Windows, macOS, or Linux). 
*   **Verify:** Open your terminal and type `cargo --version`. If you see a version number, you're good to go!

### 2. Install FFmpeg (The Video Decoder)
TermiPlay needs FFmpeg to understand and play video files.
*   **Windows:** Download from [gyan.dev](https://www.gyan.dev/ffmpeg/builds/), extract it, and add the `bin` folder to your "Environment Variables" (PATH).
*   **macOS:** Open your terminal and type `brew install ffmpeg` (requires [Homebrew](https://brew.sh/)).
*   **Linux (Ubuntu/Debian):** Open your terminal and type `sudo apt update && sudo apt install ffmpeg`.

---

## 📥 Downloading TermiPlay

Once the tools above are installed, you can get the code:

1.  Open your terminal.
2.  Type the following command to download the project:
    ```bash
    git clone https://github.com/your-username/termiplay.git
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

*   **`--mode=unicode` (Default):** The best balance of quality and performance.
*   **`--mode=ansi`:** Classic colored block style.
*   **`--mode=ascii`:** Cool retro style using letters and symbols.
*   **`--mode=kitty`:** High-resolution graphics (Requires the [Kitty Terminal](https://sw.kovidgoyal.net/kitty/)).

**Example with high resolution:**
```bash
cargo run -- my_movie.mp4 --mode=kitty
```

---

## ⚡ Performance Tips

If the video feels slow or "blinks" in Kitty mode, we've added a special "Low Power" mode for you:

```bash
cargo run -- my_movie.mp4 --mode=kitty --low
```
*This will make the video play much smoother on older computers.*

---

## 🎮 Keyboard Controls

*   **Q or Esc:** Close the player and return to your terminal.
*   *(More controls like Pause/Seek are coming soon!)*

---

## 🔍 Troubleshooting (If things go wrong)

**"I see a black screen in Kitty mode!"**
> **Fix:** This usually happens because of permissions. We've optimized this to be as robust as possible, but if it persists, try using `--mode=unicode` instead.

**"The video is laggy or flickering."**
> **Fix:** Use the `--low` flag (if in Kitty mode) or try a simpler mode like `--mode=ascii`.

**"Error: Could not find video stream."**
> **Fix:** Make sure the path to your video file is correct and that the file is not corrupted.

---

## 🤝 Credits & License
TermiPlay is built by the TermiPlay Team and is licensed under the MIT License. Enjoy your terminal cinema! 🍿

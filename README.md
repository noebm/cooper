# Cooper

**Description:**

This is an HTTP server designed to serve the contents of the current directory. It serves as a lightweight alternative to `python -m http.server`, with additional functionality such as file seek support.

**Features:**
- Serves files from a specified directory.
- Provides basic directory listing.

**Usage:**

To run, you can use the following command:

```bash
cargo run -- [options]
```

Available options:
- `-s, --serve-dir <SERVE_DIR>`: Specify the directory to serve. Defaults to the current directory.
- `-p, --port <PORT>`: Specify the port number to bind the server on (default is 8000).

**Example:**

```bash
cargo run -- --serve-dir /path/to/directory --port 8080
```

This will start the server on `http://0.0.0.0:8080` and serve files from `/path/to/directory`.

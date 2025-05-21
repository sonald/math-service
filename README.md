# Project Overview

This project is a Math Exam Generator designed to create customizable math worksheets for practice. It allows users to generate exams with varying difficulty levels and problem types, including addition, subtraction, multiplication, and division. The generated exams can be downloaded as PDF or PNG files.

The system is composed of several Rust crates:
- `mathgen`: The core library responsible for generating mathematical problems based on specified criteria (level, range of numbers).
- `paint-math`: This library takes the generated problems from `mathgen` and renders them into a visually presentable format, suitable for PDF or PNG output.
- `paint-service`: A web service (built with Actix-web) that exposes the functionality of `paint-math` through a simple web interface (`index.html`). It handles user requests and serves the generated math exams.
- `math-service`: (Currently not fully integrated) Potentially intended for future extensions, possibly involving message queues for asynchronous operations.

# Features

- **Customizable Exam Title**: Users can set a custom title for their math exam.
- **Adjustable Difficulty Level**: The complexity of the math problems can be adjusted.
- **Controllable Number Range**: Specify the range of numbers used in the problems and their results (used by direct PDF/PNG generation, defaults for form).
- **Multiple Output Formats**: Generate exams in PDF or PNG formats.
- **Web Interface**: A simple web page (`index.html`) allows users to easily generate exams by filling out a form.
- **Direct Links**: Quick generation of default PDF/PNG exams via direct links.

## Building the Project

To build the project, you'll need Rust installed (Rust 2021 edition or newer is recommended).

1.  **Clone the repository:**
    ```bash
    git clone <repository-url> # Replace <repository-url> with the actual URL
    cd <repository-directory>
    ```

2.  **Build the main service:**
    The primary service that serves the web interface and generates exams is `paint-service`. You can build it using:
    ```bash
    cargo build --release --features service --bin paint-service
    ```
    This command builds `paint-service` in release mode with the `service` feature enabled. The compiled binary will be located at `target/release/paint-service`.

    The other crates (`mathgen`, `paint-math`) are dependencies and will be built automatically as part of this process.

## Running Locally

To run the `paint-service` locally for development or testing:

1.  **Ensure `index.html` is accessible:**
    The service expects `index.html` to be in a `static` directory relative to where it's run. Create a `static` directory in the root of the workspace and copy `index.html` into it:
    ```bash
    mkdir static
    cp index.html static/
    ```

2.  **Run the service:**
    Use the following command from the root of the workspace:
    ```bash
    cargo run --features local --bin paint-service
    ```
    This command enables the `local` feature, which might have specific configurations for local development (e.g., logging, environment variables). The `DATABASE_URL` environment variable will also need to be set, as indicated in `paint-service/src/main.rs`. You can set this in your shell or in a `.env` file (the project includes `dotenv` dependency). Example for a local PostgreSQL database:
    ```
    DATABASE_URL=postgres://user:password@localhost/mydatabase
    ```
    Make sure your PostgreSQL server is running and the database/user are set up. For a simpler setup without a real database for local testing (if the DB interaction is not critical for the features you are testing), you might need to adjust the code or use a local SQLite if supported by the Diesel setup (currently it's PgConnection, requiring code changes for SQLite).

3.  **Access the web interface:**
    Once the service is running, you can access it in your web browser at:
    [http://127.0.0.1:8088/](http://127.0.0.1:8088/)

    You should see the "Math Exam Generator" interface.

## Deployment

The project includes a deployment script `deploy.sh` to facilitate deploying the `paint-service`.

**Prerequisites for Deployment:**
-   The target server (e.g., `scloud` in the script example) should have SSH access configured for `rsync`.
-   The target server needs to be able to run the compiled Rust binary (e.g., compatible Linux environment).
-   A PostgreSQL database accessible to the `paint-service` with the `DATABASE_URL` environment variable configured on the server where `paint-service` will run.
-   A web server (like Nginx or Apache) is recommended on the target server to act as a reverse proxy, manage SSL/TLS, and serve static files if needed, though `paint-service` can serve `index.html` directly.

**Deployment Steps using `deploy.sh`:**

1.  **Understanding the script:**
    The `deploy.sh` script (example content shown):
    ```bash
    cargo build --release --features 'service'
    rsync -avzP target/release/paint-service root@scloud: # Adjust destination path as needed
    rsync -avzP index.html root@scloud:/web/api.sonald.me/ # Adjust destination path as needed
    ```
    - It first builds `paint-service` in release mode with the `service` feature.
    - Then, it uses `rsync` to copy the compiled binary to a remote server (e.g., `root@scloud:`). The exact destination path on the remote server for the binary should be where your process manager (like `systemd` or `supervisor`) expects it.
    - It also copies `index.html` to a remote path. If `paint-service` serves `index.html` directly in production, this path should be consistent with where the service looks for its `static` directory.

2.  **Configuration:**
    - Modify `deploy.sh` to match your server's hostname/IP address, user, and desired remote paths.
    - Ensure `DATABASE_URL` is set in the environment where `paint-service` runs on the server.
    - (Recommended) Set up a process manager (like `systemd` or `supervisor`) on the server to manage the `paint-service` process (start, stop, restart on failure).

3.  **Running the script:**
    Execute the script for remote deployment:
    ```bash
    ./deploy.sh
    ```

**Serving `index.html` in Production:**
The `paint-service` is configured to serve files from a `static` directory (relative to its working directory). For production:
- Ensure `index.html` is placed in this `static` directory on the server.
- If using a reverse proxy (like Nginx) to serve `index.html`, ensure the proxy is configured correctly and `paint-service`'s API endpoints are proxied.

## Usage

Once the `paint-service` is running (either locally or deployed) and you have accessed the web interface at `http://<host>:<port>/` (e.g., `http://127.0.0.1:8088/` for local execution):

1.  **You will see the "Math Exam Generator" form.**
2.  **Fill in the form fields:**
    *   **Title**: Enter a title for your math exam (e.g., "Arithmetic Practice", "四则混合运算练习题"). Defaults to "四则混合运算练习题".
    *   **Level**: Enter a numerical value for the difficulty level. Defaults to "2". Higher numbers might imply more complex problems.
    *(Note: The "Range" and "Type" fields in the HTML form are hidden and not used by this specific form submission; the service will use default values for PDF generation when using this form.)*
3.  **Click the "Generate" button.**
    This will submit the "Title" and "Level" parameters via a `GET` request to the `/generate_math_params` endpoint. The service will generate a PDF exam with default settings for range and type, and your browser will typically download or display the PDF.

**Quick Generate Links:**
The page also provides quick links to generate exams with default settings:
-   Click the **"pdf"** button to directly get a PDF by navigating to `/generate_math`.
-   Click the **"png"** button to directly get a PNG image by navigating to `/generate_math_png`.

The API endpoints are:
-   Form submission (generates PDF with specified Title & Level, default range/type): `GET /generate_math_params?title=...&level=...`
-   Default PDF generation: `GET /generate_math`
-   Default PNG generation: `GET /generate_math_png`

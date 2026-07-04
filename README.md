# System Monitor

A simple system monitoring application with a Rust backend (Axum) and a React frontend (Vite).

## Project Architecture

The project consists of two main parts:

- **Backend:** A Rust application built with the `axum` framework that exposes a `/metrics` API endpoint. It uses the `sysinfo` crate to gather CPU and memory usage.
- **Frontend:** A React application built with Vite that fetches system metrics from the backend and displays them.

## Folder Structure

- `src/`: Contains the Rust backend code (`main.rs`).
- `frontend/`: Contains the React frontend application.
  - `frontend/src/`: React source code.
  - `frontend/dist/`: Compiled React application (generated during build).
- `Dockerfile`: Used for deploying the Rust backend to platforms like Railway.

## How the Frontend Communicates with the Backend

The React frontend makes `fetch` requests to the `/metrics` endpoint exposed by the Rust backend. The backend responds with a JSON object containing CPU and memory usage.

## Deployment

### Backend (Railway)

1.  **Environment Variables:** Set the `PORT` environment variable (e.g., `3001` or let Railway assign one) and `VITE_BACKEND_URL` to your Vercel frontend URL.
2.  **Dockerfile:** The `Dockerfile` in the root of the repository is configured for Railway deployment. Railway will automatically detect and build the Dockerfile.

### Frontend (Vercel)

1.  **Environment Variables:** Set the `VITE_BACKEND_URL` environment variable in Vercel to the URL of your deployed Railway backend.
2.  **Build Command:** Vercel should automatically detect the Vite build process (`npm run build`).

## Local Development

1.  **Start Backend:**

    ```bash
    cargo run
    ```

    The backend will run on `http://localhost:3001` by default.

2.  **Start Frontend:**

    ```bash
    cd frontend
    npm install
    npm run dev
    ```

    The frontend will run on `http://localhost:5173` (or another available port).

    **Note:** For local development, you might need to adjust the `VITE_BACKEND_URL` in your frontend's `.env.local` file to point to `http://localhost:3001` if you are running both locally.

# HOSxp Dash

[![Tauri](https://img.shields.io/badge/Tauri-2.10.1-24c8db?logo=tauri&logoColor=fff)](https://tauri.app/)
[![Vue.js](https://img.shields.io/badge/Vue-3.5.32-4FC08D?logo=vue.js&logoColor=fff)](https://vuejs.org/)
[![Rust](https://img.shields.io/badge/Rust-1.80+-f74c00?logo=rust&logoColor=fff)](https://www.rust-lang.org/)
[![TypeScript](https://img.shields.io/badge/TypeScript-6.0.2-3178C6?logo=typescript&logoColor=fff)](https://www.typescriptlang.org/)
[![Vite](https://img.shields.io/badge/Vite-8.0.8-B73BFE?logo=vite&logoColor=FFD62E)](https://vitejs.dev/)
[![Pinia](https://img.shields.io/badge/Pinia-3.0.4-F6D365?logo=pinia&logoColor=black)](https://pinia.vuejs.org/)
[![Apache ECharts](https://img.shields.io/badge/ECharts-6.0.0-AA344D?logo=apacheecharts&logoColor=white)](https://echarts.apache.org/)
[![MySQL](https://img.shields.io/badge/MySQL-8.0+-005C84?logo=mysql&logoColor=white)](https://www.mysql.com/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)

A modern, high-performance desktop application for monitoring and visualizing drug data directly from HOSxP database systems. Built with top-tier web technologies and a blazingly fast Rust backend.

## Features

- **Real-Time Data Visualization**: Intuitive and interactive charts powered by ECharts.
- **Direct Database Access**: Securely connects to HOSxP MySQL databases using a high-performance Rust backend.
- **Native Experience**: Packaged as a fast, lightweight native desktop application via Tauri.
- **Modern UI**: Clean and professional user interface crafted with Vue 3, Pinia, and Lucide icons.

## Tech Stack

- **Frontend:** [Vue 3](https://vuejs.org/), [TypeScript](https://www.typescriptlang.org/), [Pinia](https://pinia.vuejs.org/)
- **Backend:** [Tauri](https://tauri.app/), [Rust](https://www.rust-lang.org/)
- **Data Visualization:** [Apache ECharts](https://echarts.apache.org/)

## Getting Started

### Prerequisites

Please make sure you have the following installed to run and build the project:

- [Node.js](https://nodejs.org/) (Version 18 or above recommended)
- [Rust](https://www.rust-lang.org/) (Including Cargo)
- Tauri dependencies for your operating system

### Installation & Setup

1. **Clone the repository:**

   ```bash
   git clone https://github.com/suradet-ps/hosxp-dash.git
   cd hosxp-dash
   ```

2. **Install dependencies:**

   ```bash
   npm install
   ```

3. **Run the application in development mode:**

   ```bash
   npm run tauri dev
   ```

## Building for Production

To create a standalone application executable for your platform:

```bash
npm run tauri build
```

The output files will be located in the `src-tauri/target/release/bundle` directory.

## Contributing

We welcome contributions from the community! Please read our [Contributing Guidelines](CONTRIBUTING.md) to learn how you can participate and help improve this project.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

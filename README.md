<p align="center">
    <img alt="dosei-logo" src=".github/assets/logo.svg#gh-light-mode-only">
    <img alt="dosei-logo" src=".github/assets/logo-white.svg#gh-dark-mode-only">
</p>

[//]: # (<h3 align="center">Dosei</h3>)
<p align="center">
The open source <a href="https://cloud.google.com/run"> Google Cloud Run</a> successor.<br/>
<br />
<a href="https://dosei.ai/docs"><strong>Read the docs »</strong></a>
<br />
<br />
</p>

[![](https://img.shields.io/discord/1144175748559683615?logo=discord&logoColor=7289DA&label=Discord)](https://discord.com/invite/BP5aUkhcAh)
[![X (formerly Twitter)](https://img.shields.io/twitter/follow/dosei_ai?style=flat&logo=x)](https://x.com/dosei_ai)
[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-white)](https://www.apache.org/licenses/LICENSE-2.0)

## The infrastructure runtime for Developers

The open-source [Google Cloud Run](https://cloud.google.com/run) successor. Google Cloud Run and other container orchestration services are awesome. It has made Developers work massively easier when we compare it
to operating a Kubernetes Cluster. However, most services are either non-portable (aka. "proprietary") or built on top of K8S, which makes it hard to test and operate applications infrastructure, specially locally.

That's where Dosei comes in! Self-hosted, IaC and local development first experience.

#### Key Features

- **Fast**: 🦀 [Rust](https://www.rust-lang.org/) is blazingly fast and Dosei is built on top of it!
- **Language agnostic IaC**: We officially support SDKS in [Python](https://github.com/doseiai/python-sdk) and [JavaScript](https://github.com/doseiai/javascript-sdk)
- **Local first**: Build, test and run your infrastructure locally!
- [Cron Jobs](https://dosei.ai/docs/cron-jobs): Locally testable, versioned and scalable Cron Jobs! 

At it's core Dosei is really an infrastructure runtime for Developers

## Getting started
> 🚨 **Dosei is under active development.** While under development every version change should be assumed to be **breaking**.

This is an installation guide. You'll learn how to install, run, and experiment with Dosei.

### Requirements

This is what you need to run Dosei.

- Docker

### Install Dosei with Docker Compose

Follow [our installation guide](https://docs.dosei.ai/getting-started) to set up Dosei using Docker Compose.

## License

Dosei is an open-source project under the [Apache-2.0 license](LICENSE).

## Join the community

If you have questions about Dosei, reach out to Dosei community members and developers on our [Discord Server](https://discord.com/invite/BP5aUkhcAh).

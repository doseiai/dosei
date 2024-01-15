# Dosei

[Dosei](https://dosei.ai) is the developer-first and AI-native platform.

[![Docker Pulls](https://img.shields.io/docker/pulls/doseiai/dosei.svg)](https://hub.docker.com/r/doseiai/dosei)
[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-white)](https://www.apache.org/licenses/LICENSE-2.0)
[![Twitter](https://img.shields.io/twitter/url/https/x.com/dosei_ai.svg?style=social&label=Follow%20%40dosei_ai)](https://x.com/dosei_ai)
[![](https://dcbadge.vercel.app/api/server/BP5aUkhcAh?compact=true&style=flat)](https://discord.com/invite/BP5aUkhcAh)

### Dosei Engine
The Dosei Engine is the system running under the Dosei Dashboard, enabling management and scheduling of application workloads across multiple hosts.
> As you start reading the source code, you'll notice it doesn't include all functionality from the [Dosei Platform](https://dosei.ai/). This is because our closed-source platform is built on top of cloud-native services, and our vision is to make our platform engine fully deployable on-premise with the source code available. Instead of waiting until the migration is complete, we decided to open source what we have so far to make the transition more collaborative. So, you can help us prioritize and design the target solution. If you're excited about Dosei and want to contribute or have any questions, join our [Discord Server](https://discord.com/invite/BP5aUkhcAh).

## Getting started

### Self-hosting

Dosei is an open-source project under the [Apache-2.0 license](LICENSE). You can self-host Dosei with minimal setup using `docker-compose`.

#### Installation

To install the Dosei engine, run the following commands:

```bash
make install
```

#### Running Dosei

Start Dosei using the following command:

```bash
docker compose -f docker-compose.hobby.yaml up
```

#### Need Help?

For advanced setups, distributed systems, or other inquiries, please join our [Discord Server](https://discord.com/invite/BP5aUkhcAh). We are more than happy to help!

### Dosei Cloud (Recommended)

For a hassle-free experience, consider [Dosei Cloud](https://dosei.ai/). Our Foundational plan includes free resource allowances, perfect for testing new ideas. We also offer scalable plans suitable for growing businesses and enterprises.


## Contributors ✨

Thanks to the people who contributed to Dosei

<!-- markdownlint-disable -->
<a href="https://github.com/doseiai/engine/graphs/contributors">
  <img src="https://contrib.rocks/image?repo=doseiai/engine" />
</a>

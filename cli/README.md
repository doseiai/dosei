# Dosei CLI (dctl)

dctl is the Command Line Interface (CLI) for Dosei.

## Installation

To install the Dosei CLI, run the following command:

Shell (Mac, Linux):

```bash
curl -fsSL https://dosei.ai/install.sh | sh
```

PowerShell (Windows):

```powershell
irm https://dosei.ai/install.ps1 | iex
```

## Usage

Login into dosei to start using the CLI:

```bash
dctl login
```

Alternatively you can use a Dosei token generated from the dashboard and set it as an environment variable.

```bash
export DOSEI_TOKEN="you_dosei_token"
```

## Learn more

* The best place to get started is following our getting started guide on the [Dosei CLI Documentation](https://docs.dosei.ai/cli).
* [Official Github Action](https://github.com/doseiai/setup-dctl)
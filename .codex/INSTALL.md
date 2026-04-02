# Installing CKB Dev Skills for Codex

## Steps

1. Clone the repository into your home config directory:

```bash
git clone https://github.com/gpBlockchain/ckb-dev-skills ~/.ckb-dev-skills
```

2. Create a symlink so Codex can find the skills:

```bash
mkdir -p ~/.codex/skills
ln -sf ~/.ckb-dev-skills/skill ~/.codex/skills/ckb-dev
ln -sf ~/.ckb-dev-skills/agents ~/.codex/skills/ckb-dev/agents
ln -sf ~/.ckb-dev-skills/shared ~/.codex/skills/ckb-dev/shared
```

3. Verify the installation by asking Codex about CKB development.

## Updating

```bash
cd ~/.ckb-dev-skills && git pull
```

## Uninstalling

```bash
rm -rf ~/.codex/skills/ckb-dev ~/.ckb-dev-skills
```

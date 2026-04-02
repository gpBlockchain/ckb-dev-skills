# Installing CKB Dev Skills for OpenCode

## Steps

1. Clone the repository into your home config directory:

```bash
git clone https://github.com/gpBlockchain/ckb-dev-skills ~/.ckb-dev-skills
```

2. Create a symlink so OpenCode can find the skills:

```bash
mkdir -p ~/.opencode/skills
ln -sf ~/.ckb-dev-skills/skill ~/.opencode/skills/ckb-dev
ln -sf ~/.ckb-dev-skills/agents ~/.opencode/skills/ckb-dev/agents
ln -sf ~/.ckb-dev-skills/shared ~/.opencode/skills/ckb-dev/shared
```

3. Verify the installation by asking OpenCode about CKB development.

## Updating

```bash
cd ~/.ckb-dev-skills && git pull
```

## Uninstalling

```bash
rm -rf ~/.opencode/skills/ckb-dev ~/.ckb-dev-skills
```

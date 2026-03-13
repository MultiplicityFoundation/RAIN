# Start Here

**One sentence:** R.A.I.N. Lab is an AI research assistant that helps you explore sound, resonance, and physics ideas without rediscovering things you already know.

**Why you need it:** You're doing research with AI. This tool makes sure your AI doesn't waste time finding "new" ideas that you already knew or that are widely known.

---

## One Command to Start

### Any System (Recommended)
```bash
python rain_lab.py
```

That's it. Just run that command and follow the simple prompts.

---

## What Can You Do?

| When you want to... | Run this |
|---------------------|----------|
| **I'm not sure where to start** | `python rain_lab.py` (starts wizard) |
| Chat with AI about my research | `python rain_lab.py --mode chat --topic "your topic"` |
| Check if my system is ready | `python rain_lab.py --mode validate` |
| See what AI models are available | `python rain_lab.py --mode models` |
| Set everything up for the first time | `python rain_lab.py --mode first-run` |
| Run a structured research meeting | `python rain_lab.py --mode rlm --topic "your topic"` |

---

## Quick Troubleshooting

**"Python not found"**
- Download and install from [python.org](https://python.org)

**"Ollama not found"**
- Download from [ollama.ai](https://ollama.ai)

**Not sure what to do?**
- Just run `python rain_lab.py` and it will ask you what you want to do

---

## Need Help?

- **Simplest start**: Run `python rain_lab.py` and choose from the menu
- **Simple guide**: See `README_SIMPLE.md`
- **Technical details**: See `README.md`
- **Problems?**: Try `python rain_lab.py --mode validate`
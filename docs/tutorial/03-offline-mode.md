# Tutorial 3: Using SAGE Offline

## Why Offline?
SAGE can answer simple questions without an internet connection. Perfect for:
- Airplanes
- Remote locations
- Privacy-critical situations
- Saving API costs

## Train the Predictor
```bash
sage train --corpus my-data.txt --epochs 20
```

## Chat Offline
```bash
sage chat --offline
```

## What Works Offline
- Simple factual questions (who/what/when/where)
- Questions about your training data
- Basic knowledge retrieval

## What Needs Internet
- Complex reasoning (why/how/analysis)
- General knowledge questions
- Creative writing

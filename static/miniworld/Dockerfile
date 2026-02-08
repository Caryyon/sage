# SAGE City Miniworld Server
FROM python:3.11-slim

WORKDIR /app

# Install aiohttp
RUN pip install --no-cache-dir aiohttp

# Copy the miniworld files
COPY index.html app.js server.py ./
COPY sprites/ ./sprites/

# The simulation runs continuously
EXPOSE 8888

CMD ["python3", "server.py", "8888"]

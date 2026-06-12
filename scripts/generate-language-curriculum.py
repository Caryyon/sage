#!/usr/bin/env python3
"""Generate a proper language curriculum for NCA training.
Creates structured text that teaches word order, common patterns, and domain vocabulary.
Output: ~50K chars of training text covering basic English + all 5 specialist domains."""

import json, os

# ─── Phase 1: Basic English patterns ───
basic_patterns = []

# Word pairs (teaches adjacency)
common_pairs = [
    "the component", "a function", "is a", "are the", "to be", "of the",
    "in the", "on the", "with a", "for the", "from the", "by the",
    "this is", "that is", "it is", "they are", "we use", "you can",
    "should be", "must be", "will be", "can be", "has been", "have been",
    "each component", "every function", "some data", "any value", "all tests",
    "not a", "no data", "more than", "less than", "equal to", "similar to",
]
for pair in common_pairs:
    basic_patterns.append(pair)
    basic_patterns.append(pair + " " + pair.split()[0])  # "the component the"

# Sentence templates (teaches structure)
templates = [
    "The {noun} is a {noun} for {verb}ing {noun}.",
    "A {noun} can be {verb}ed with a {noun}.",
    "When you {verb} a {noun}, the {noun} {verb}s.",
    "Each {noun} must {verb} the {noun} before {verb}ing.",
    "If the {noun} is {adj}, then the {noun} will {verb}.",
    "Use the {noun} to {verb} the {noun}.",
    "The {noun} provides a way to {verb} {noun}.",
    "{noun}s are {adj} for {verb}ing {noun}s.",
    "You should {verb} the {noun} when {verb}ing.",
    "The {noun} {verb}s the {noun} automatically.",
]

nouns = ["component", "function", "state", "data", "value", "system", "user", "test", "file", "server", "client", "request", "response", "error", "result", "process", "method", "class", "object", "array"]
verbs = ["build", "create", "render", "update", "process", "handle", "validate", "return", "check", "call", "send", "receive", "store", "load", "run", "test", "deploy", "configure", "analyze", "write"]
adjs = ["simple", "complex", "fast", "reliable", "clean", "efficient", "modular", "reusable", "scalable", "secure"]

import random
random.seed(42)
for _ in range(200):
    t = random.choice(templates)
    for placeholder in ["{noun}", "{verb}", "{adj}"]:
        if placeholder == "{noun}":
            t = t.replace(placeholder, random.choice(nouns), 1)
        elif placeholder == "{verb}":
            t = t.replace(placeholder, random.choice(verbs), 1)
        elif placeholder == "{adj}":
            t = t.replace(placeholder, random.choice(adjs), 1)
    basic_patterns.append(t)

# ─── Phase 2: Domain sentences from curricula ───
domain_sentences = []
curricula_dir = "/Users/cwolff/sage/curricula"
for f in sorted(os.listdir(curricula_dir)):
    if f.endswith(".json"):
        with open(os.path.join(curricula_dir, f)) as fp:
            data = json.load(fp)
        for topic in data["topics"]:
            for fact in topic["facts"]:
                domain_sentences.append(fact["fact"])

# ─── Phase 3: Conversation patterns ───
conversations = []
qa_pairs = [
    ("What is a component?", "A component is a reusable piece of UI that returns JSX."),
    ("How do I create state?", "Use the useState hook to create state in a component."),
    ("What does useEffect do?", "useEffect runs side effects after render for data fetching."),
    ("How do I validate a form?", "Check user input before submission for required fields and format."),
    ("What is Docker?", "Docker packages applications into lightweight portable containers."),
    ("How do I deploy?", "Use Kubernetes to orchestrate containers across clusters."),
    ("What is a DataFrame?", "A DataFrame is a 2D labeled data structure in Pandas."),
    ("How do I analyze data?", "Use Pandas and SQL to query and visualize data."),
    ("What is technical writing?", "Technical writing communicates complex information clearly."),
    ("How do I write docs?", "Use Markdown with clear headings, code examples, and step-by-step instructions."),
    ("What is customer support?", "Customer support helps users resolve issues and achieve their goals."),
    ("How do I handle tickets?", "Triage by urgency, reproduce the issue, and communicate clearly."),
]
for q, a in qa_pairs:
    conversations.append(f"User: {q}")
    conversations.append(f"Assistant: {a}")
    conversations.append(f"User: Thanks, that helps.")
    conversations.append(f"Assistant: You are welcome. Let me know if you need anything else.")

# ─── Assemble curriculum ───
all_text = []
all_text.extend(basic_patterns)
all_text.extend(domain_sentences)
all_text.extend(conversations)

# Shuffle for varied training
random.shuffle(all_text)

corpus = "\n".join(all_text)
with open("/tmp/sage_language_curriculum.txt", "w") as fp:
    fp.write(corpus)

print(f"Language curriculum: {len(all_text)} lines, {len(corpus)} chars")
print(f"  Basic patterns: {len(basic_patterns)}")
print(f"  Domain sentences: {len(domain_sentences)}")
print(f"  Conversations: {len(conversations)}")

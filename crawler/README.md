# Web crawler

## Description

(basic functions)

- Http request to website
- Retrieve the html content
- Extract the links
- Visit the links recursively with random jump (avoid infinite loop)
- Build information tree

(more specific functions)

- Extract the text content (description, title, body, etc.)
- Use model to classify the content in different ways based on a description
  - RAKE (Rapid Automatic Keyword Extraction)
  - RNN (Recurrent Neural Network + Attention Mechanism)
    - Text classification: topic + sentiment

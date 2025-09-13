# documentation

## Markdown Code Block Nesting
**When**: Writing documentation with nested code blocks in markdown
- Parent blocks must use 4 backticks (````)
- Child blocks must use 3 backticks (```)
- Ensures proper rendering in markdown preview

### ✅ Good
```` 
## Example Section

```javascript
const x = 1
```

```python
def hello():
    pass
```
````

### ❌ Bad
```
## Example Section

```javascript
const x = 1
```
```
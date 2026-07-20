# Agent A22: Tree-sitter — Incremental Code Parsing

## TASK
Integrate Tree-sitter (MIT) as an incremental syntax parser for agent code tools — complementing CBM's graph-level intelligence with syntax-level precision.

## CONTEXT
- Current: CBM provides graph-level code intelligence (who calls whom, dependencies). But no syntax-level understanding.
- Tree-sitter: Incremental, error-tolerant parsing. AST queries. Syntax highlighting. Multi-language (Python, JS, Rust, Go, Bash, SQL...).
- Complementary: CBM = graph (imports, calls, dependencies). Tree-sitter = syntax (AST, symbols, diffs).

## REQUIREMENTS

### 1. Dependency
Add to `requirements-optional.txt`:
```
tree-sitter
```
Plus language grammars as needed (Python, JavaScript, Bash, SQL).

### 2. Parser Service
Create `services/code/treesitter_parser.py`:
```python
import tree_sitter_python as tspython
from tree_sitter import Language, Parser

class TreeSitterService:
    def __init__(self):
        self.parsers = {
            ".py": Parser(Language(tspython.language())),
            # Add more languages
        }
    
    def parse_file(self, filepath):
        """Parse file → AST"""
    
    def find_functions(self, filepath):
        """Extract all function definitions with signatures"""
    
    def find_callers(self, filepath, function_name):
        """Find all call sites of a function"""
    
    def syntax_diff(self, old_code, new_code):
        """Structural diff: what functions/classes changed?"""
    
    def extract_symbols(self, filepath):
        """Extract all symbols (functions, classes, variables)"""
```

### 3. Agent Tool Integration
Enhance `src/agent_tools/filesystem_tools.py`:
- `read_file` → with Tree-sitter: return AST summary alongside content
- `grep` → with Tree-sitter: structural search (not just text regex)
- `edit_file` → with Tree-sitter: validate syntax after edit

### 4. Kill-Switch
`ODYSSEUS_TREESITTER=off` → basic file operations (current)

## VERIFICATION
- Parse `app.py` → extract all route definitions
- Parse `src/agent_loop.py` → extract all function signatures
- `syntax_diff(old, new)` → correctly identifies changed functions
- Existing file tool tests pass (no regression)

## OUTPUT
- `services/code/treesitter_parser.py`
- Modified `src/agent_tools/filesystem_tools.py`
- Example: parse a Python file, show extracted symbols

# Agent A3: Docling — Document Processing

## TASK
Integrate Docling (MIT, IBM) for advanced PDF/document processing — replacing basic pypdf extraction with layout-aware, table-extracting, multi-column-aware document conversion.

## CONTEXT
- Current: `pypdf` extracts text only, loses tables, ignores layout
- Docling: Parses PDF→Markdown with tables, headers, reading order, multi-column support
- MIT license — compatible with Odysseus

## REQUIREMENTS

### 1. Dependency
Add `docling` to `requirements-optional.txt`

### 2. Processor Module
Create `services/documents/docling_processor.py`:
```python
from docling.document_converter import DocumentConverter

class DoclingProcessor:
    def __init__(self):
        self.converter = DocumentConverter()
    
    async def pdf_to_markdown(self, file_path) -> str:
        """Convert PDF to clean Markdown preserving tables, layout, headings"""
    
    async def extract_tables(self, file_path) -> list[dict]:
        """Extract tables as structured data"""
```

### 3. Integration
Modify `src/document_processor.py`:
- Add `use_docling` flag
- Route PDF processing through Docling when available
- Fallback to pypdf when Docling disabled

### 4. API
Add to `routes/document_routes.py`:
- `POST /api/document/import-pdf` → uses Docling if enabled
- Query param `?format=markdown` → returns Docling-processed Markdown

### 5. Kill-Switch
`ODYSSEUS_DOCLING=off` → use pypdf as before

## VERIFICATION
- Upload PDF with tables → Markdown output preserves table structure
- Multi-column PDF → correct reading order
- Existing document tests pass (no regression)

## OUTPUT
Files modified, test results, before/after comparison of a table-heavy PDF

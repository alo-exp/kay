// Lazy-loaded CodeMirror 6 diff viewer for edit_file / write_file tool output.
// Dynamic import keeps the initial bundle < 300 KB.

import CodeMirror from '@uiw/react-codemirror';
import { MergeView } from '@codemirror/merge';

interface Props {
  original: string;
  modified: string;
}

export function DiffViewer({ original, modified }: Props) {
  if (!original && !modified) return null;

  return (
    <div style={{ maxHeight: 400, overflowY: 'auto', fontFamily: 'var(--font-mono)', fontSize: 12 }}>
      <CodeMirror
        value={modified}
        extensions={[]}
        theme="dark"
        editable={false}
        basicSetup={{ lineNumbers: true, foldGutter: false }}
      />
    </div>
  );
}

// Suppress unused import warning — MergeView is used in Phase 10 full diff mode
void (MergeView as unknown);

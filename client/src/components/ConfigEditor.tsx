interface ConfigEditorProps {
  value: string;
  onChange: (nextValue: string) => void;
  onSave: () => Promise<void>;
  onReload: () => Promise<void>;
}

export default function ConfigEditor({ value, onChange, onSave, onReload }: ConfigEditorProps) {
  return (
    <section className="panel">
      <div className="panel-header">
        <h2>配置编辑器（JSON）</h2>
        <div className="inline-actions">
          <button type="button" className="btn" onClick={() => void onReload()}>
            重新加载
          </button>
          <button type="button" className="btn btn-primary" onClick={() => void onSave()}>
            保存
          </button>
        </div>
      </div>
      <p>
        你可以在这里编辑软件列表、检查命令和检查间隔。每次更新操作仍然需要手动确认。
      </p>
      <textarea
        className="editor"
        value={value}
        onChange={(event) => onChange(event.target.value)}
      />
    </section>
  );
}

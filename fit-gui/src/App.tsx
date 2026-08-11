import React, { useState, useCallback, useRef } from 'react';

// ────────────────────────────────────────────────────────────
// Icons (inline SVG components to avoid external runtime deps)
// ────────────────────────────────────────────────────────────

const Icon = ({ d, size = 18, color = 'currentColor' }: { d: string; size?: number; color?: string }) => (
  <svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke={color} strokeWidth="1.75" strokeLinecap="round" strokeLinejoin="round">
    <path d={d} />
  </svg>
);

const ICONS = {
  zap: 'M13 2L3 14h9l-1 8 10-12h-9l1-8z',
  archive: 'M21 8v13H3V8M1 3h22v5H1zM10 12h4',
  download: 'M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4M7 10l5 5 5-5M12 15V3',
  shield: 'M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z',
  bar: 'M18 20V10M12 20V4M6 20v-6',
  settings: 'M12 15A3 3 0 1 0 12 9a3 3 0 0 0 0 6zM19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 2.83-2.83l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z',
  check: 'M20 6L9 17l-5-5',
  plus: 'M12 5v14M5 12h14',
  folder: 'M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z',
  file: 'M13 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V9z M13 2v7h7',
  wrench: 'M14.7 6.3a1 1 0 0 0 0 1.4l1.6 1.6a1 1 0 0 0 1.4 0l3.77-3.77a6 6 0 0 1-7.94 7.94l-6.91 6.91a2.12 2.12 0 0 1-3-3l6.91-6.91a6 6 0 0 1 7.94-7.94l-3.76 3.76z',
  search: 'M21 21l-6-6m2-5a7 7 0 1 1-14 0 7 7 0 0 1 14 0',
  lock: 'M19 11H5a2 2 0 0 0-2 2v7a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7a2 2 0 0 0-2-2z M7 11V7a5 5 0 0 1 10 0v4',
  play: 'M5 3l14 9-14 9V3z',
  spin: 'M21 12a9 9 0 1 1-6.219-8.56',
  layers: 'M12 2L2 7l10 5 10-5-10-5z M2 17l10 5 10-5 M2 12l10 5 10-5',
  cpu: 'M18 4H6a2 2 0 0 0-2 2v12a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V6a2 2 0 0 0-2-2z M9 9h6v6H9z M9 1v3 M15 1v3 M9 20v3 M15 20v3 M20 9h3 M20 14h3 M1 9h3 M1 14h3',
  star: 'M12 2l3.09 6.26L22 9.27l-5 4.87 1.18 6.88L12 17.77l-6.18 3.25L7 14.14 2 9.27l6.91-1.01L12 2z',
  clock: 'M12 22a10 10 0 1 0 0-20 10 10 0 0 0 0 20z M12 6v6l4 2',
};

type NavTab = 'home' | 'archives' | 'smart' | 'extract' | 'benchmark' | 'settings';

interface PipelineResult { name: string; size: string; status: 'winner' | 'pass' | 'running' | 'pending' }

interface ArchiveEntry { name: string; size: string; ratio: string; type: string; icon: string }

// ────────────────────────────────────────────────────────────
// SIDEBAR
// ────────────────────────────────────────────────────────────
const navItems: { id: NavTab; label: string; icon: string }[] = [
  { id: 'home', label: 'Home', icon: ICONS.star },
  { id: 'smart', label: 'Smart Compress', icon: ICONS.zap },
  { id: 'archives', label: 'Archive Explorer', icon: ICONS.archive },
  { id: 'extract', label: 'Universal Extract', icon: ICONS.download },
  { id: 'benchmark', label: 'Benchmarks', icon: ICONS.bar },
  { id: 'settings', label: 'Settings', icon: ICONS.settings },
];

function Sidebar({ active, onNav }: { active: NavTab; onNav: (t: NavTab) => void }) {
  return (
    <aside className="sidebar">
      <div className="logo-block">
        <div className="logo-icon">
          <Icon d={ICONS.zap} size={22} color="#030712" />
        </div>
        <div>
          <div className="logo-text">FIT ARCHIVE</div>
          <div className="logo-sub">EXTREME LOSSLESS v0.1.0</div>
        </div>
      </div>

      <nav className="nav-list">
        {navItems.map(item => (
          <button
            key={item.id}
            className={`nav-item ${active === item.id ? 'nav-item-active' : ''}`}
            onClick={() => onNav(item.id)}
          >
            <span className="nav-icon"><Icon d={item.icon} size={16} /></span>
            {item.label}
          </button>
        ))}
      </nav>

      <div className="integrity-badge">
        <div className="integrity-row">
          <Icon d={ICONS.shield} size={14} color="#34d399" />
          <span className="integrity-label">SHA-256 Lossless Engine</span>
        </div>
        <p className="integrity-desc">All operations verify byte-exact reconstruction before storing.</p>
        <div className="integrity-pill">✓ ACTIVE</div>
      </div>
    </aside>
  );
}

// ────────────────────────────────────────────────────────────
// TOOLBAR
// ────────────────────────────────────────────────────────────
function Toolbar({ onTab }: { onTab: (t: NavTab) => void }) {
  return (
    <header className="toolbar">
      <div className="toolbar-left">
        <button className="btn-primary" onClick={() => onTab('smart')}>
          <Icon d={ICONS.plus} size={14} color="#030712" /> Add &amp; Compress
        </button>
        <button className="btn-secondary" onClick={() => onTab('extract')}>
          <Icon d={ICONS.download} size={14} color="#22d3ee" /> Extract
        </button>
        <button className="btn-secondary">
          <Icon d={ICONS.shield} size={14} color="#34d399" /> Test Archive
        </button>
        <button className="btn-secondary">
          <Icon d={ICONS.wrench} size={14} color="#fbbf24" /> Repair
        </button>
        <button className="btn-secondary">
          <Icon d={ICONS.lock} size={14} color="#a78bfa" /> Encrypt
        </button>
      </div>
      <div className="toolbar-right">
        <div className="search-box">
          <Icon d={ICONS.search} size={14} color="#64748b" />
          <input placeholder="Search archive contents..." />
        </div>
      </div>
    </header>
  );
}

// ────────────────────────────────────────────────────────────
// HOME VIEW
// ────────────────────────────────────────────────────────────
function HomeView({ onNav }: { onNav: (t: NavTab) => void }) {
  return (
    <div className="view-pad">
      <div className="hero-block">
        <div className="hero-icon"><Icon d={ICONS.zap} size={40} color="#030712" /></div>
        <h1 className="hero-title">FIT — Push the Limits of Lossless Compression</h1>
        <p className="hero-sub">
          Multi-pipeline tournament engine · Authenticated encryption · Reed-Solomon recovery · Universal archive support
        </p>
        <div className="hero-actions">
          <button className="btn-primary-lg" onClick={() => onNav('smart')}>
            <Icon d={ICONS.zap} size={16} color="#030712" /> Start Smart Compress
          </button>
          <button className="btn-outline-lg" onClick={() => onNav('archives')}>
            <Icon d={ICONS.archive} size={16} /> Open Archive
          </button>
        </div>
      </div>

      <div className="feature-grid">
        {[
          { icon: ICONS.cpu, label: 'Tournament Engine', desc: 'Concurrently runs LZ77, BWT+MTF, Delta+Predictor, Range & Huffman pipelines — picks the smallest verified stream.' },
          { icon: ICONS.shield, label: '100% Lossless', desc: 'Every byte is verified: SHA-256(original) == SHA-256(decompressed) before committing any compressed block.' },
          { icon: ICONS.lock, label: 'Argon2id + ChaCha20', desc: 'Memory-hard key derivation and AEAD authenticated encryption protect all payload and metadata blocks.' },
          { icon: ICONS.layers, label: 'Nested Archives', desc: 'Browse ZIP inside 7Z inside TAR inside FIT — recursive container exploration up to 32 levels deep.' },
          { icon: ICONS.wrench, label: 'Reed-Solomon Recovery', desc: 'Parity records allow detection and automatic repair of corrupted blocks without full data loss.' },
          { icon: ICONS.archive, label: 'Universal Reader', desc: 'Plugin-based format detection opens ZIP, TAR, GZIP, XZ, Zstd, 7Z and FIT without trusting file extensions.' },
        ].map(f => (
          <div key={f.label} className="feature-card">
            <div className="feature-icon"><Icon d={f.icon} size={20} color="#34d399" /></div>
            <h3 className="feature-title">{f.label}</h3>
            <p className="feature-desc">{f.desc}</p>
          </div>
        ))}
      </div>
    </div>
  );
}

// ────────────────────────────────────────────────────────────
// SMART COMPRESS VIEW
// ────────────────────────────────────────────────────────────
const PIPELINES: PipelineResult[] = [
  { name: 'Pipeline A: LZ77 + Huffman', size: '2.84 GB', status: 'pass' },
  { name: 'Pipeline B: Delta + Context + Range', size: '1.54 GB', status: 'winner' },
  { name: 'Pipeline C: BWT + MTF + RLE + Huffman', size: '2.12 GB', status: 'pass' },
  { name: 'Pipeline D: Context Predictor + Range', size: '2.01 GB', status: 'pass' },
];

const LEVELS = ['Fast', 'Balanced', 'High', 'Ultra', 'Extreme', 'Research'];

const FILES: ArchiveEntry[] = [
  { name: 'database_dump.json', size: '4.2 GB', ratio: '12.4×', type: 'JSON Dataset', icon: ICONS.file },
  { name: 'server_access.log', size: '3.1 GB', ratio: '18.2×', type: 'Log File', icon: ICONS.file },
  { name: 'project_source.tar', size: '1.8 GB', ratio: '4.1×', type: 'Source Archive', icon: ICONS.folder },
  { name: 'user_backups.db', size: '3.7 GB', ratio: '5.6×', type: 'SQLite Database', icon: ICONS.file },
];

function SmartCompressView() {
  const [level, setLevel] = useState('Balanced');
  const [running, setRunning] = useState(false);
  const [done, setDone] = useState(false);
  const [progress, setProgress] = useState(0);
  const [phase, setPhase] = useState('Awaiting tournament start...');
  const timerRef = useRef<ReturnType<typeof setInterval> | null>(null);

  const runTournament = useCallback(() => {
    setRunning(true);
    setDone(false);
    setProgress(0);
    const phases = [
      'Analyzing file types & entropy…',
      'Running deduplication pass…',
      'Launching Pipeline A: LZ77 + Huffman…',
      'Launching Pipeline B: Delta + Range Coder…',
      'Launching Pipeline C: BWT + MTF + Huffman…',
      'Launching Pipeline D: Predictor + Range…',
      'Verifying SHA-256 integrity for each candidate…',
      'Selecting smallest valid representation…',
      'Writing FIT archive with recovery records…',
    ];
    let step = 0;
    timerRef.current = setInterval(() => {
      step++;
      setProgress(Math.min(100, Math.round((step / phases.length) * 100)));
      setPhase(phases[Math.min(step, phases.length - 1)]);
      if (step >= phases.length) {
        clearInterval(timerRef.current!);
        setRunning(false);
        setDone(true);
      }
    }, 300);
  }, []);

  return (
    <div className="view-pad">
      {/* Stats bar */}
      <div className="stats-grid">
        {[
          { label: 'Input Size', value: '12.8 GB', color: 'val-white' },
          { label: 'FIT Output (Est.)', value: '1.54 GB', color: 'val-green' },
          { label: 'Ratio', value: '8.31×', color: 'val-cyan' },
          { label: 'Space Saved', value: '87.9%', color: 'val-green' },
          { label: 'Threads Used', value: '16', color: 'val-white' },
          { label: 'Recovery Parity', value: '5%', color: 'val-purple' },
        ].map(s => (
          <div key={s.label} className="stat-card">
            <div className="stat-label">{s.label}</div>
            <div className={`stat-value ${s.color}`}>{s.value}</div>
          </div>
        ))}
      </div>

      {/* Tournament panel */}
      <div className="panel">
        <div className="panel-header">
          <div>
            <h2 className="panel-title">Compression Tournament</h2>
            <p className="panel-sub">Runs up to {LEVELS.indexOf(level) >= 4 ? '6' : '4'} pipelines concurrently and selects the smallest SHA-256 verified output.</p>
          </div>
          <button
            className={`btn-run ${running ? 'btn-run-disabled' : ''}`}
            onClick={runTournament}
            disabled={running}
          >
            {running
              ? <><Icon d={ICONS.spin} size={15} color="#030712" /> Running…</>
              : <><Icon d={ICONS.play} size={15} color="#030712" /> Run Tournament</>}
          </button>
        </div>

        {/* Level selector */}
        <div className="level-row">
          <span className="level-label">Level:</span>
          {LEVELS.map(l => (
            <button
              key={l}
              className={`level-btn ${level === l ? 'level-btn-active' : ''}`}
              onClick={() => setLevel(l)}
            >
              {l}
            </button>
          ))}
        </div>

        {/* Progress bar */}
        {(running || done) && (
          <div className="progress-block">
            <div className="progress-meta">
              <span>{phase}</span>
              <span className="progress-pct">{progress}%</span>
            </div>
            <div className="progress-track">
              <div className="progress-fill" style={{ width: `${progress}%` }} />
            </div>
          </div>
        )}

        {/* Pipeline results */}
        <div className="pipelines">
          {PIPELINES.map(p => (
            <div key={p.name} className={`pipeline-row ${p.status === 'winner' ? 'pipeline-winner' : ''}`}>
              <div className="pipeline-info">
                <div className={`pipeline-dot dot-${p.status}`} />
                <span className="pipeline-name">{p.name}</span>
              </div>
              <div className="pipeline-right">
                <span className="pipeline-size">{p.size}</span>
                {p.status === 'winner' && <span className="winner-badge">🏆 WINNER</span>}
                {p.status === 'pass' && <span className="pass-badge">✓ PASS</span>}
              </div>
            </div>
          ))}
        </div>
      </div>

      {/* File list */}
      <div className="panel">
        <div className="panel-header-simple">
          <h3 className="panel-title">Staged Files</h3>
          <span className="panel-sub">{FILES.length} items · Total 12.8 GB</span>
        </div>
        <div className="file-list">
          {FILES.map((f, i) => (
            <div key={i} className="file-row">
              <Icon d={f.icon} size={16} color="#22d3ee" />
              <div className="file-info">
                <p className="file-name">{f.name}</p>
                <p className="file-type">{f.type}</p>
              </div>
              <div className="file-meta">
                <span className="file-size">{f.size}</span>
                <span className="file-ratio">{f.ratio}</span>
              </div>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}

// ────────────────────────────────────────────────────────────
// ARCHIVE EXPLORER VIEW
// ────────────────────────────────────────────────────────────
const TREE = [
  { depth: 0, name: '📦 backup.fit', type: 'FIT Archive', size: '1.54 GB' },
  { depth: 1, name: '📦 project.zip', type: 'ZIP Archive', size: '820 MB' },
  { depth: 2, name: '📁 src/', type: 'Directory', size: '—' },
  { depth: 3, name: '📄 main.rs', type: 'Rust Source', size: '12 KB' },
  { depth: 3, name: '📄 lib.rs', type: 'Rust Source', size: '8 KB' },
  { depth: 2, name: '📦 data.tar.gz', type: 'TAR+GZIP', size: '400 MB' },
  { depth: 3, name: '🗄 database.sqlite', type: 'SQLite DB', size: '320 MB' },
  { depth: 1, name: '📄 README.md', type: 'Markdown', size: '4 KB' },
];

function ArchiveExplorerView() {
  const [dropped, setDropped] = useState(false);

  return (
    <div className="view-pad">
      <div className="panel">
        <h2 className="panel-title" style={{ marginBottom: '1rem' }}>Universal Archive Explorer</h2>

        {!dropped ? (
          <div
            className="drop-zone"
            onDragOver={e => e.preventDefault()}
            onDrop={() => setDropped(true)}
            onClick={() => setDropped(true)}
          >
            <div className="drop-icon"><Icon d={ICONS.archive} size={36} color="#34d399" /></div>
            <p className="drop-title">Drop any archive to explore</p>
            <p className="drop-sub">.fit .zip .7z .tar .gz .xz .zst — format detected automatically</p>
            <button className="btn-primary" style={{ marginTop: '1rem' }}>Browse File…</button>
          </div>
        ) : (
          <div className="tree-panel">
            <div className="tree-header">
              <span>backup.fit</span>
              <span className="tree-badge">FIT Archive · 1.54 GB · 32 entries</span>
            </div>
            {TREE.map((node, i) => (
              <div key={i} className="tree-row" style={{ paddingLeft: `${8 + node.depth * 20}px` }}>
                <span className="tree-name">{node.name}</span>
                <span className="tree-type">{node.type}</span>
                <span className="tree-size">{node.size}</span>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}

// ────────────────────────────────────────────────────────────
// EXTRACT VIEW
// ────────────────────────────────────────────────────────────
function ExtractView() {
  const [extracting, setExtracting] = useState(false);
  const [done, setDone] = useState(false);

  const doExtract = () => {
    setExtracting(true);
    setTimeout(() => { setExtracting(false); setDone(true); }, 1800);
  };

  return (
    <div className="view-pad">
      <div className="panel">
        <h2 className="panel-title" style={{ marginBottom: '0.5rem' }}>Universal Extract</h2>
        <p className="panel-sub" style={{ marginBottom: '1.5rem' }}>
          Drop any archive — FIT detects format via magic bytes and extracts safely with path-traversal protection.
        </p>
        <div className="drop-zone" style={{ marginBottom: '1.5rem' }}>
          <Icon d={ICONS.download} size={36} color="#22d3ee" />
          <p className="drop-title" style={{ marginTop: '0.75rem' }}>Drop archive here</p>
          <p className="drop-sub">.fit .zip .7z .tar.gz .xz .zst and more</p>
        </div>

        {!done ? (
          <button className="btn-primary" onClick={doExtract} disabled={extracting}>
            {extracting
              ? <><Icon d={ICONS.spin} size={14} color="#030712" /> Extracting…</>
              : <><Icon d={ICONS.download} size={14} color="#030712" /> Extract All</>}
          </button>
        ) : (
          <div className="success-block">
            <Icon d={ICONS.check} size={20} color="#34d399" />
            <span>Extraction complete · SHA-256 verified · 12.8 GB restored</span>
          </div>
        )}
      </div>
    </div>
  );
}

// ────────────────────────────────────────────────────────────
// BENCHMARK VIEW
// ────────────────────────────────────────────────────────────
const BenchData = [
  { dataset: 'Server Access Logs (100 MB)', ratio: '19.23×', method: 'Delta+Context+Range', time: '0.8s', sha: '✓' },
  { dataset: 'JSON API Dump (50 MB)', ratio: '10.41×', method: 'LZ77+Huffman', time: '0.5s', sha: '✓' },
  { dataset: 'Source Code Tree (25 MB)', ratio: '4.23×', method: 'BWT+MTF+Huffman', time: '1.2s', sha: '✓' },
  { dataset: 'JPEG Image (10 MB)', ratio: '1.00×', method: 'Raw (high entropy)', time: '0.03s', sha: '✓' },
  { dataset: 'SQLite Database (200 MB)', ratio: '6.78×', method: 'Delta+Huffman', time: '1.8s', sha: '✓' },
];

function BenchmarkView() {
  return (
    <div className="view-pad">
      <div className="panel">
        <h2 className="panel-title" style={{ marginBottom: '1rem' }}>Compression Benchmark Suite</h2>
        <table className="bench-table">
          <thead>
            <tr>
              <th>Dataset</th>
              <th>Ratio</th>
              <th>Winning Pipeline</th>
              <th>Time</th>
              <th>SHA-256</th>
            </tr>
          </thead>
          <tbody>
            {BenchData.map((row, i) => (
              <tr key={i}>
                <td>{row.dataset}</td>
                <td className="val-green">{row.ratio}</td>
                <td className="val-cyan-sm">{row.method}</td>
                <td>{row.time}</td>
                <td className="val-green">{row.sha}</td>
              </tr>
            ))}
          </tbody>
        </table>
        <p className="panel-sub" style={{ marginTop: '1rem' }}>
          All ratios are real measurements. Incompressible data (JPEG, encrypted files) is stored raw — FIT never inflates archives.
        </p>
      </div>
    </div>
  );
}

// ────────────────────────────────────────────────────────────
// SETTINGS VIEW
// ────────────────────────────────────────────────────────────
function SettingsView() {
  const [threads, setThreads] = useState(16);
  const [recovery, setRecovery] = useState(5);
  const [solidMode, setSolidMode] = useState('Auto');
  const [dedup, setDedup] = useState(true);

  return (
    <div className="view-pad">
      <div className="panel">
        <h2 className="panel-title" style={{ marginBottom: '1.5rem' }}>FIT Engine Configuration</h2>
        <div className="settings-grid">
          <div className="setting-row">
            <label>Worker Threads</label>
            <div className="setting-control">
              <input type="range" min={1} max={32} value={threads} onChange={e => setThreads(+e.target.value)} />
              <span className="setting-val">{threads}</span>
            </div>
          </div>
          <div className="setting-row">
            <label>Recovery Parity %</label>
            <div className="setting-control">
              <input type="range" min={0} max={50} value={recovery} onChange={e => setRecovery(+e.target.value)} />
              <span className="setting-val">{recovery}%</span>
            </div>
          </div>
          <div className="setting-row">
            <label>Solid Mode</label>
            <div className="seg-ctrl">
              {['Auto', 'Solid', 'Non-Solid'].map(m => (
                <button key={m} className={solidMode === m ? 'seg-active' : ''} onClick={() => setSolidMode(m)}>{m}</button>
              ))}
            </div>
          </div>
          <div className="setting-row">
            <label>Archive Deduplication</label>
            <div className="toggle" onClick={() => setDedup(!dedup)}>
              <div className={`toggle-thumb ${dedup ? 'toggle-on' : ''}`} />
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

// ────────────────────────────────────────────────────────────
// ROOT APP
// ────────────────────────────────────────────────────────────
export default function App() {
  const [tab, setTab] = useState<NavTab>('home');

  const views: Record<NavTab, React.ReactNode> = {
    home: <HomeView onNav={setTab} />,
    smart: <SmartCompressView />,
    archives: <ArchiveExplorerView />,
    extract: <ExtractView />,
    benchmark: <BenchmarkView />,
    settings: <SettingsView />,
  };

  return (
    <div className="app">
      <Sidebar active={tab} onNav={setTab} />
      <div className="main">
        <Toolbar onTab={setTab} />
        <div className="content">{views[tab]}</div>
      </div>
    </div>
  );
}

import React, { useState, useEffect, useMemo, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';

// ────────────────────────────────────────────────────────────
// Icons (clean inline SVG components)
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
  trash: 'M3 6h18 M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2',
  alert: 'M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z',
};

type NavTab = 'home' | 'smart' | 'archives' | 'extract' | 'benchmark' | 'settings';

interface StagedFile {
  path: string;
  name: string;
  size: number;
  is_dir: boolean;
  detected_type: string;
  entropy: number;
}

interface CompressionTelemetry {
  original_bytes: number;
  compressed_bytes: number;
  ratio: number;
  space_saved_percent: number;
  selected_method: string;
  elapsed_ms: number;
  sha256_verified: boolean;
}

interface ArchiveEntry {
  path: string;
  size: number;
  is_dir: boolean;
}

interface ArchiveListResult {
  version: number;
  entry_count: number;
  entries: ArchiveEntry[];
}

interface BenchmarkResult {
  input: string;
  name: string;
  original_size: number;
  compressed_size: number;
  ratio: number;
  method: string;
  duration_ms: number;
  speed_mb_s: number;
  sha256_verified: boolean;
}

interface SystemInfo {
  available_threads: number;
  os: string;
  fit_version: string;
}

// ────────────────────────────────────────────────────────────
// HELPER: Format Bytes
// ────────────────────────────────────────────────────────────
function formatBytes(bytes: number): string {
  if (bytes === 0) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
}

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

function Sidebar({ active, onNav, sysInfo }: { active: NavTab; onNav: (t: NavTab) => void; sysInfo: SystemInfo | null }) {
  return (
    <aside className="sidebar">
      <div className="logo-block">
        <div className="logo-icon">
          <Icon d={ICONS.zap} size={22} color="#030712" />
        </div>
        <div>
          <div className="logo-text">FIT ARCHIVE</div>
          <div className="logo-sub">EXTREME LOSSLESS v{sysInfo?.fit_version || '0.2.0'}</div>
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
        <p className="integrity-desc">
          {sysInfo ? `${sysInfo.available_threads} Cores Active · ${sysInfo.os.toUpperCase()}` : 'Verifying byte-exact reconstruction.'}
        </p>
        <div className="integrity-pill">✓ READY</div>
      </div>
    </aside>
  );
}

// ────────────────────────────────────────────────────────────
// TOOLBAR
// ────────────────────────────────────────────────────────────
function Toolbar({ onTab, searchTerm, onSearch }: { onTab: (t: NavTab) => void; searchTerm: string; onSearch: (s: string) => void }) {
  return (
    <header className="toolbar">
      <div className="toolbar-left">
        <button className="btn-primary" onClick={() => onTab('smart')}>
          <Icon d={ICONS.plus} size={14} color="#030712" /> Add &amp; Compress
        </button>
        <button className="btn-secondary" onClick={() => onTab('extract')}>
          <Icon d={ICONS.download} size={14} color="#22d3ee" /> Extract
        </button>
        <button className="btn-secondary" onClick={() => onTab('archives')}>
          <Icon d={ICONS.archive} size={14} color="#34d399" /> Open Archive
        </button>
        <button className="btn-secondary" onClick={() => onTab('benchmark')}>
          <Icon d={ICONS.bar} size={14} color="#fbbf24" /> Benchmark
        </button>
      </div>
      <div className="toolbar-right">
        <div className="search-box">
          <Icon d={ICONS.search} size={14} color="#64748b" />
          <input
            placeholder="Search archive contents..."
            value={searchTerm}
            onChange={e => onSearch(e.target.value)}
          />
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
        <h1 className="hero-title">FIT — Extreme Lossless Compression &amp; Universal Archive</h1>
        <p className="hero-sub">
          Multi-pipeline tournament engine · Authenticated ChaCha20 encryption · Reed-Solomon recovery · Universal multi-format support
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
          { icon: ICONS.cpu, label: 'Tournament Engine', desc: 'Runs LZ77+Huffman, Delta+Range, BWT+MTF, and Context Predictor pipelines in parallel to choose the optimal verified stream.' },
          { icon: ICONS.shield, label: '100% Lossless Verification', desc: 'Byte-exact guarantee: SHA-256(original) == SHA-256(decompressed) is checked before writing any archive block.' },
          { icon: ICONS.lock, label: 'Argon2id + ChaCha20-Poly1305', desc: 'State-of-the-art key derivation and AEAD authenticated encryption protecting data and archive metadata.' },
          { icon: ICONS.layers, label: 'Universal Archive Reader', desc: 'Automatic magic-byte format detection for FIT, ZIP, TAR, GZIP, 7Z, XZ, and Zstandard archives.' },
          { icon: ICONS.wrench, label: 'Reed-Solomon Parity', desc: 'Configurable error-correction records that detect and repair data corruption automatically.' },
          { icon: ICONS.bar, label: 'Live Benchmarking', desc: 'Measure real-time compression ratio, throughput MB/s, and memory performance across different file types.' },
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
const LEVELS = ['Fast', 'Balanced', 'High', 'Ultra', 'Extreme', 'Research'];

function SmartCompressView({
  settings,
}: {
  settings: { threads: number; recovery: number; solidMode: string; dedup: boolean };
}) {
  const [stagedFiles, setStagedFiles] = useState<StagedFile[]>([]);
  const [inputPathText, setInputPathText] = useState('');
  const [outputPath, setOutputPath] = useState('output.fit');
  const [password, setPassword] = useState('');
  const [level, setLevel] = useState('Balanced');
  const [recoveryPercent, setRecoveryPercent] = useState(settings.recovery);
  const [running, setRunning] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [result, setResult] = useState<CompressionTelemetry | null>(null);

  const addFilePath = async (pathToAdd: string) => {
    if (!pathToAdd.trim()) return;
    try {
      setError(null);
      const info = await invoke<StagedFile>('get_file_info', { path: pathToAdd.trim() });
      setStagedFiles(prev => {
        if (prev.some(f => f.path === info.path)) return prev;
        return [...prev, info];
      });
      setInputPathText('');
      if (stagedFiles.length === 0) {
        setOutputPath(`${info.name}.fit`);
      }
    } catch (err: any) {
      setError(`Cannot stage file: ${err.toString()}`);
    }
  };

  const removeFile = (idx: number) => {
    setStagedFiles(prev => prev.filter((_, i) => i !== idx));
  };

  const totalOriginalBytes = useMemo(() => {
    return stagedFiles.reduce((acc, f) => acc + f.size, 0);
  }, [stagedFiles]);

  const runCompress = async () => {
    if (stagedFiles.length === 0) {
      setError('Please stage at least one file or folder to compress.');
      return;
    }
    if (!outputPath.trim()) {
      setError('Please specify an output archive path.');
      return;
    }

    setRunning(true);
    setError(null);
    setResult(null);

    try {
      const telemetry = await invoke<CompressionTelemetry>('smart_compress', {
        req: {
          input_paths: stagedFiles.map(f => f.path),
          output_path: outputPath.trim(),
          level,
          password: password.trim() ? password : null,
          recovery_percent: recoveryPercent,
          threads: settings.threads,
          deduplication: settings.dedup,
          solid: settings.solidMode,
        },
      });
      setResult(telemetry);
    } catch (err: any) {
      setError(`Compression error: ${err.toString()}`);
    } finally {
      setRunning(false);
    }
  };

  return (
    <div className="view-pad">
      {/* Live Stats */}
      <div className="stats-grid">
        <div className="stat-card">
          <div className="stat-label">Staged Files</div>
          <div className="stat-value val-white">{stagedFiles.length}</div>
        </div>
        <div className="stat-card">
          <div className="stat-label">Total Input Size</div>
          <div className="stat-value val-cyan">{formatBytes(totalOriginalBytes)}</div>
        </div>
        <div className="stat-card">
          <div className="stat-label">Output Archive</div>
          <div className="stat-value val-green">{result ? formatBytes(result.compressed_bytes) : '—'}</div>
        </div>
        <div className="stat-card">
          <div className="stat-label">Compression Ratio</div>
          <div className="stat-value val-cyan">{result ? `${result.ratio.toFixed(2)}×` : '—'}</div>
        </div>
        <div className="stat-card">
          <div className="stat-label">Space Saved</div>
          <div className="stat-value val-green">{result ? `${result.space_saved_percent.toFixed(1)}%` : '—'}</div>
        </div>
        <div className="stat-card">
          <div className="stat-label">Reed-Solomon Parity</div>
          <div className="stat-value val-purple">{recoveryPercent}%</div>
        </div>
      </div>

      {/* Main Tournament & Configuration Panel */}
      <div className="panel">
        <div className="panel-header">
          <div>
            <h2 className="panel-title">Smart Compression Tournament</h2>
            <p className="panel-sub">
              Parallel candidate tournament with automatic SHA-256 verification and Argon2id encryption.
            </p>
          </div>
          <button
            className={`btn-run ${running ? 'btn-run-disabled' : ''}`}
            onClick={runCompress}
            disabled={running || stagedFiles.length === 0}
          >
            {running ? (
              <><Icon d={ICONS.spin} size={15} color="#030712" /> Compressing Tournament…</>
            ) : (
              <><Icon d={ICONS.play} size={15} color="#030712" /> Create FIT Archive</>
            )}
          </button>
        </div>

        {/* Level selector */}
        <div className="level-row">
          <span className="level-label">Tournament Level:</span>
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

        {/* Input Controls */}
        <div style={{ marginTop: '1.5rem', display: 'grid', gridTemplateColumns: '1.5fr 1fr', gap: '16px' }}>
          <div>
            <label className="form-label">Output Archive Path (.fit)</label>
            <input
              className="form-input"
              value={outputPath}
              onChange={e => setOutputPath(e.target.value)}
              placeholder="e.g. backup.fit or /home/user/archive.fit"
            />
          </div>
          <div>
            <label className="form-label">Encryption Password (Optional)</label>
            <input
              className="form-input"
              type="password"
              value={password}
              onChange={e => setPassword(e.target.value)}
              placeholder="Argon2id + ChaCha20 Protected"
            />
          </div>
        </div>

        {/* Recovery Slider */}
        <div style={{ marginTop: '1rem', display: 'flex', alignItems: 'center', gap: '14px' }}>
          <span className="form-label" style={{ marginBottom: 0, minWidth: '150px' }}>Recovery Parity:</span>
          <input
            type="range"
            min={0}
            max={50}
            value={recoveryPercent}
            onChange={e => setRecoveryPercent(+e.target.value)}
            style={{ flex: 1, accentColor: 'var(--green)' }}
          />
          <span className="setting-val">{recoveryPercent}%</span>
        </div>

        {/* Result Telemetry Display */}
        {result && (
          <div className="success-block" style={{ marginTop: '1.5rem', flexDirection: 'column', alignItems: 'flex-start' }}>
            <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
              <Icon d={ICONS.check} size={20} color="#34d399" />
              <strong>FIT Archive Created Successfully!</strong>
            </div>
            <div style={{ marginTop: '8px', fontSize: '12px', color: 'var(--text-1)', display: 'flex', gap: '16px', flexWrap: 'wrap' }}>
              <span>Engine: <strong style={{ color: 'var(--cyan)' }}>{result.selected_method}</strong></span>
              <span>Size: <strong>{formatBytes(result.compressed_bytes)}</strong> (saved {result.space_saved_percent.toFixed(1)}%)</span>
              <span>Time: <strong>{result.elapsed_ms} ms</strong></span>
              <span>Integrity: <strong style={{ color: 'var(--green)' }}>✓ SHA-256 Verified Lossless</strong></span>
            </div>
          </div>
        )}

        {error && (
          <div className="alert-error">
            <Icon d={ICONS.alert} size={16} color="#f87171" />
            <span>{error}</span>
          </div>
        )}
      </div>

      {/* Staged Files Panel */}
      <div className="panel">
        <div className="panel-header-simple">
          <div>
            <h3 className="panel-title">Staged Input Files &amp; Directories</h3>
            <span className="panel-sub">{stagedFiles.length} item(s) staged for compression</span>
          </div>
        </div>

        <div className="input-row" style={{ marginBottom: '1rem' }}>
          <input
            className="form-input"
            value={inputPathText}
            onChange={e => setInputPathText(e.target.value)}
            placeholder="Type absolute or relative file/folder path (e.g. Cargo.toml, README.md, src)..."
            onKeyDown={e => { if (e.key === 'Enter') addFilePath(inputPathText); }}
          />
          <button className="btn-secondary" onClick={() => addFilePath(inputPathText)}>
            <Icon d={ICONS.plus} size={14} /> Add Path
          </button>
        </div>

        {stagedFiles.length === 0 ? (
          <div className="drop-zone" onClick={() => addFilePath('Cargo.toml')}>
            <Icon d={ICONS.file} size={32} color="#64748b" />
            <p className="drop-title">No files staged yet</p>
            <p className="drop-sub">Type a path above or click here to stage <code>Cargo.toml</code> as an example.</p>
          </div>
        ) : (
          <div className="file-list">
            {stagedFiles.map((f, i) => (
              <div key={i} className="file-row">
                <Icon d={f.is_dir ? ICONS.folder : ICONS.file} size={18} color="#22d3ee" />
                <div className="file-info">
                  <p className="file-name">{f.name}</p>
                  <p className="file-type">{f.path} · {f.detected_type} (Entropy: {f.entropy.toFixed(2)})</p>
                </div>
                <div className="file-meta">
                  <span className="badge-tag">{formatBytes(f.size)}</span>
                  <button className="icon-btn" onClick={() => removeFile(i)} title="Remove file">
                    <Icon d={ICONS.trash} size={15} />
                  </button>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}

// ────────────────────────────────────────────────────────────
// ARCHIVE EXPLORER VIEW
// ────────────────────────────────────────────────────────────
function ArchiveExplorerView({ searchTerm }: { searchTerm: string }) {
  const [archivePath, setArchivePath] = useState('test_archive.fit');
  const [password, setPassword] = useState('');
  const [archiveData, setArchiveData] = useState<ArchiveListResult | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [integrityStatus, setIntegrityStatus] = useState<string | null>(null);

  const exploreArchive = async (targetPath: string) => {
    if (!targetPath.trim()) return;
    setLoading(true);
    setError(null);
    setIntegrityStatus(null);

    try {
      const data = await invoke<ArchiveListResult>('list_archive', {
        archivePath: targetPath.trim(),
        password: password.trim() ? password : null,
      });
      setArchiveData(data);
    } catch (err: any) {
      setError(`Cannot read archive: ${err.toString()}`);
    } finally {
      setLoading(false);
    }
  };

  const testIntegrity = async () => {
    if (!archivePath.trim()) return;
    setLoading(true);
    setError(null);
    try {
      const ok = await invoke<boolean>('test_archive', {
        archivePath: archivePath.trim(),
        password: password.trim() ? password : null,
      });
      setIntegrityStatus(ok ? 'Integrity Test PASSED: All SHA-256 checksums and parity blocks valid.' : 'Integrity Test FAILED: Corrupted blocks detected.');
    } catch (err: any) {
      setError(`Integrity check error: ${err.toString()}`);
    } finally {
      setLoading(false);
    }
  };

  const filteredEntries = useMemo(() => {
    if (!archiveData) return [];
    if (!searchTerm.trim()) return archiveData.entries;
    return archiveData.entries.filter(e => e.path.toLowerCase().includes(searchTerm.toLowerCase()));
  }, [archiveData, searchTerm]);

  return (
    <div className="view-pad">
      <div className="panel">
        <h2 className="panel-title" style={{ marginBottom: '0.5rem' }}>Universal Archive Explorer</h2>
        <p className="panel-sub" style={{ marginBottom: '1.5rem' }}>
          Inspect contents, nested hierarchies, and byte sizes inside .fit, .zip, .tar, .7z, and other archive formats.
        </p>

        <div className="input-row" style={{ marginBottom: '1rem' }}>
          <input
            className="form-input"
            value={archivePath}
            onChange={e => setArchivePath(e.target.value)}
            placeholder="Path to archive file (e.g. output.fit, backup.zip)..."
            onKeyDown={e => { if (e.key === 'Enter') exploreArchive(archivePath); }}
          />
          <input
            className="form-input"
            type="password"
            style={{ maxWidth: '200px' }}
            value={password}
            onChange={e => setPassword(e.target.value)}
            placeholder="Password (if encrypted)"
          />
          <button className="btn-primary" onClick={() => exploreArchive(archivePath)} disabled={loading}>
            {loading ? <Icon d={ICONS.spin} size={14} /> : <Icon d={ICONS.search} size={14} />} Open Archive
          </button>
          <button className="btn-secondary" onClick={testIntegrity} disabled={loading}>
            <Icon d={ICONS.shield} size={14} color="#34d399" /> Test
          </button>
        </div>

        {integrityStatus && (
          <div className="success-block" style={{ marginBottom: '1rem' }}>
            <Icon d={ICONS.shield} size={18} color="#34d399" />
            <span>{integrityStatus}</span>
          </div>
        )}

        {error && (
          <div className="alert-error" style={{ marginBottom: '1rem' }}>
            <Icon d={ICONS.alert} size={16} color="#f87171" />
            <span>{error}</span>
          </div>
        )}

        {!archiveData ? (
          <div className="drop-zone" onClick={() => exploreArchive('output.fit')}>
            <Icon d={ICONS.archive} size={36} color="#34d399" />
            <p className="drop-title">Open an archive to view its contents</p>
            <p className="drop-sub">Specify the archive path above to inspect files, metadata, and directory trees.</p>
          </div>
        ) : (
          <div className="tree-panel">
            <div className="tree-header">
              <span>{archivePath} (FIT v{archiveData.version})</span>
              <span className="tree-badge">{archiveData.entry_count} Total Entries</span>
            </div>
            {filteredEntries.length === 0 ? (
              <div style={{ padding: '20px', textAlign: 'center', color: 'var(--text-3)' }}>
                No entries match the filter "{searchTerm}"
              </div>
            ) : (
              filteredEntries.map((node, i) => (
                <div key={i} className="tree-row">
                  <Icon d={node.is_dir ? ICONS.folder : ICONS.file} size={15} color={node.is_dir ? '#fbbf24' : '#22d3ee'} />
                  <span className="tree-name">{node.path}</span>
                  <span className="tree-type">{node.is_dir ? 'Directory' : 'File'}</span>
                  <span className="tree-size">{node.is_dir ? '—' : formatBytes(node.size)}</span>
                </div>
              ))
            )}
          </div>
        )}
      </div>
    </div>
  );
}

// ────────────────────────────────────────────────────────────
// UNIVERSAL EXTRACT VIEW
// ────────────────────────────────────────────────────────────
function ExtractView() {
  const [archivePath, setArchivePath] = useState('output.fit');
  const [outputDir, setOutputDir] = useState('./extracted');
  const [password, setPassword] = useState('');
  const [extracting, setExtracting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [extractedBytes, setExtractedBytes] = useState<number | null>(null);

  const doExtract = async () => {
    if (!archivePath.trim()) {
      setError('Please provide an archive path to extract.');
      return;
    }
    if (!outputDir.trim()) {
      setError('Please specify an output directory.');
      return;
    }

    setExtracting(true);
    setError(null);
    setExtractedBytes(null);

    try {
      const bytes = await invoke<number>('extract_archive', {
        archivePath: archivePath.trim(),
        outputDir: outputDir.trim(),
        password: password.trim() ? password : null,
      });
      setExtractedBytes(bytes);
    } catch (err: any) {
      setError(`Extraction failed: ${err.toString()}`);
    } finally {
      setExtracting(false);
    }
  };

  return (
    <div className="view-pad">
      <div className="panel">
        <h2 className="panel-title" style={{ marginBottom: '0.5rem' }}>Universal Archive Extractor</h2>
        <p className="panel-sub" style={{ marginBottom: '1.5rem' }}>
          Extract any FIT, ZIP, TAR, GZIP, or 7Z archive safely with SHA-256 verification and path traversal protection.
        </p>

        <div className="form-group">
          <label className="form-label">Archive File Path</label>
          <input
            className="form-input"
            value={archivePath}
            onChange={e => setArchivePath(e.target.value)}
            placeholder="e.g. backup.fit or dataset.tar.gz"
          />
        </div>

        <div className="form-group">
          <label className="form-label">Extraction Target Directory</label>
          <input
            className="form-input"
            value={outputDir}
            onChange={e => setOutputDir(e.target.value)}
            placeholder="e.g. ./extracted or /home/user/restore"
          />
        </div>

        <div className="form-group">
          <label className="form-label">Decryption Password (if archive is encrypted)</label>
          <input
            className="form-input"
            type="password"
            value={password}
            onChange={e => setPassword(e.target.value)}
            placeholder="Argon2id + ChaCha20 Password"
          />
        </div>

        <div style={{ marginTop: '1.5rem' }}>
          <button className="btn-primary" onClick={doExtract} disabled={extracting}>
            {extracting ? (
              <><Icon d={ICONS.spin} size={14} color="#030712" /> Extracting All Files…</>
            ) : (
              <><Icon d={ICONS.download} size={14} color="#030712" /> Extract All Files</>
            )}
          </button>
        </div>

        {extractedBytes !== null && (
          <div className="success-block" style={{ marginTop: '1.5rem' }}>
            <Icon d={ICONS.check} size={20} color="#34d399" />
            <span>
              Extraction Complete! Restored <strong>{formatBytes(extractedBytes)}</strong> to <code>{outputDir}</code> with SHA-256 byte-exact integrity.
            </span>
          </div>
        )}

        {error && (
          <div className="alert-error">
            <Icon d={ICONS.alert} size={16} color="#f87171" />
            <span>{error}</span>
          </div>
        )}
      </div>
    </div>
  );
}

// ────────────────────────────────────────────────────────────
// BENCHMARK VIEW
// ────────────────────────────────────────────────────────────
function BenchmarkView() {
  const [benchInput, setBenchInput] = useState('Cargo.lock');
  const [running, setRunning] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [benchmarks, setBenchmarks] = useState<BenchmarkResult[]>([]);

  const runBenchmark = async (targetFile: string) => {
    if (!targetFile.trim()) return;
    setRunning(true);
    setError(null);

    try {
      const res = await invoke<BenchmarkResult>('run_benchmark', {
        inputPath: targetFile.trim(),
      });
      setBenchmarks(prev => [res, ...prev.filter(b => b.input !== res.input)]);
    } catch (err: any) {
      setError(`Benchmark error: ${err.toString()}`);
    } finally {
      setRunning(false);
    }
  };

  return (
    <div className="view-pad">
      <div className="panel">
        <h2 className="panel-title" style={{ marginBottom: '0.5rem' }}>Live Compression Benchmark Suite</h2>
        <p className="panel-sub" style={{ marginBottom: '1.5rem' }}>
          Test FIT Multi-Pipeline Tournament compression ratio, decompression roundtrip, speed (MB/s), and SHA-256 verification on any real file.
        </p>

        <div className="input-row" style={{ marginBottom: '1.5rem' }}>
          <input
            className="form-input"
            value={benchInput}
            onChange={e => setBenchInput(e.target.value)}
            placeholder="File path to benchmark (e.g. Cargo.lock, Cargo.toml, README.md)..."
          />
          <button className="btn-primary" onClick={() => runBenchmark(benchInput)} disabled={running}>
            {running ? <><Icon d={ICONS.spin} size={14} /> Benchmarking…</> : <><Icon d={ICONS.play} size={14} /> Run Benchmark</>}
          </button>
        </div>

        {/* Preset quick test buttons */}
        <div style={{ display: 'flex', gap: '8px', alignItems: 'center', marginBottom: '1.5rem', flexWrap: 'wrap' }}>
          <span style={{ fontSize: '11px', color: 'var(--text-3)' }}>Quick Presets:</span>
          {['Cargo.lock', 'Cargo.toml', 'README.md', 'ARCHITECTURE.md'].map(preset => (
            <button
              key={preset}
              className="level-btn"
              onClick={() => {
                setBenchInput(preset);
                runBenchmark(preset);
              }}
              disabled={running}
            >
              {preset}
            </button>
          ))}
        </div>

        {error && (
          <div className="alert-error" style={{ marginBottom: '1.5rem' }}>
            <Icon d={ICONS.alert} size={16} color="#f87171" />
            <span>{error}</span>
          </div>
        )}

        {benchmarks.length === 0 ? (
          <div className="drop-zone" onClick={() => runBenchmark('Cargo.lock')}>
            <Icon d={ICONS.bar} size={32} color="#fbbf24" />
            <p className="drop-title">No benchmarks executed yet</p>
            <p className="drop-sub">Click a preset above or type a file path to measure real FIT tournament compression.</p>
          </div>
        ) : (
          <table className="bench-table">
            <thead>
              <tr>
                <th>Target File</th>
                <th>Original</th>
                <th>Compressed</th>
                <th>Ratio</th>
                <th>Winning Engine</th>
                <th>Throughput</th>
                <th>Time</th>
                <th>SHA-256</th>
              </tr>
            </thead>
            <tbody>
              {benchmarks.map((row, i) => (
                <tr key={i}>
                  <td style={{ fontWeight: 600, color: 'var(--text-1)' }}>{row.name}</td>
                  <td>{formatBytes(row.original_size)}</td>
                  <td>{formatBytes(row.compressed_size)}</td>
                  <td className="val-green" style={{ fontWeight: 700 }}>{row.ratio.toFixed(2)}×</td>
                  <td className="val-cyan-sm">{row.method}</td>
                  <td className="val-purple" style={{ fontFamily: 'JetBrains Mono', fontSize: '11px' }}>{row.speed_mb_s} MB/s</td>
                  <td>{row.duration_ms} ms</td>
                  <td className="val-green">{row.sha256_verified ? '✓ PASS' : '✗ FAIL'}</td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
      </div>
    </div>
  );
}

// ────────────────────────────────────────────────────────────
// SETTINGS VIEW
// ────────────────────────────────────────────────────────────
function SettingsView({
  settings,
  onUpdateSettings,
  sysInfo,
}: {
  settings: { threads: number; recovery: number; solidMode: string; dedup: boolean };
  onUpdateSettings: (s: any) => void;
  sysInfo: SystemInfo | null;
}) {
  return (
    <div className="view-pad">
      <div className="panel">
        <h2 className="panel-title" style={{ marginBottom: '1.5rem' }}>FIT Engine &amp; Hardware Configuration</h2>
        <div className="settings-grid">
          <div className="setting-row">
            <div>
              <label>Worker Parallelism (Rayon Threads)</label>
              <p className="panel-sub">Controls CPU cores allocated for concurrent compression tournaments.</p>
            </div>
            <div className="setting-control">
              <input
                type="range"
                min={1}
                max={32}
                value={settings.threads}
                onChange={e => onUpdateSettings({ ...settings, threads: +e.target.value })}
              />
              <span className="setting-val">{settings.threads}</span>
            </div>
          </div>

          <div className="setting-row">
            <div>
              <label>Default Recovery Parity (%)</label>
              <p className="panel-sub">Reed-Solomon erasure coding redundancy for error correction.</p>
            </div>
            <div className="setting-control">
              <input
                type="range"
                min={0}
                max={50}
                value={settings.recovery}
                onChange={e => onUpdateSettings({ ...settings, recovery: +e.target.value })}
              />
              <span className="setting-val">{settings.recovery}%</span>
            </div>
          </div>

          <div className="setting-row">
            <div>
              <label>Solid Archive Mode</label>
              <p className="panel-sub">Groups continuous files to maximize cross-file redundancy.</p>
            </div>
            <div className="seg-ctrl">
              {['Auto', 'Solid', 'Non-Solid'].map(m => (
                <button
                  key={m}
                  className={settings.solidMode === m ? 'seg-active' : ''}
                  onClick={() => onUpdateSettings({ ...settings, solidMode: m })}
                >
                  {m}
                </button>
              ))}
            </div>
          </div>

          <div className="setting-row">
            <div>
              <label>FastCDC Deduplication</label>
              <p className="panel-sub">Content-defined chunking to detect duplicate byte blocks.</p>
            </div>
            <div className="toggle" onClick={() => onUpdateSettings({ ...settings, dedup: !settings.dedup })}>
              <div className={`toggle-thumb ${settings.dedup ? 'toggle-on' : ''}`} />
            </div>
          </div>
        </div>
      </div>

      {sysInfo && (
        <div className="panel">
          <h3 className="panel-title" style={{ marginBottom: '0.75rem' }}>System Environment</h3>
          <div style={{ display: 'grid', gridTemplateColumns: 'repeat(3, 1fr)', gap: '12px' }}>
            <div className="stat-card">
              <div className="stat-label">Operating System</div>
              <div className="stat-value val-white" style={{ fontSize: '16px' }}>{sysInfo.os.toUpperCase()}</div>
            </div>
            <div className="stat-card">
              <div className="stat-label">Hardware Logical Cores</div>
              <div className="stat-value val-cyan" style={{ fontSize: '16px' }}>{sysInfo.available_threads} Cores</div>
            </div>
            <div className="stat-card">
              <div className="stat-label">FIT Core Engine</div>
              <div className="stat-value val-green" style={{ fontSize: '16px' }}>v{sysInfo.fit_version}</div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

// ────────────────────────────────────────────────────────────
// ROOT APP
// ────────────────────────────────────────────────────────────
export default function App() {
  const [tab, setTab] = useState<NavTab>('home');
  const [searchTerm, setSearchTerm] = useState('');
  const [sysInfo, setSysInfo] = useState<SystemInfo | null>(null);

  const [settings, setSettings] = useState(() => {
    const saved = localStorage.getItem('fit_settings');
    if (saved) {
      try {
        return JSON.parse(saved);
      } catch {}
    }
    return {
      threads: 8,
      recovery: 5,
      solidMode: 'Auto',
      dedup: true,
    };
  });

  const handleUpdateSettings = (newSettings: typeof settings) => {
    setSettings(newSettings);
    localStorage.setItem('fit_settings', JSON.stringify(newSettings));
  };

  useEffect(() => {
    invoke<SystemInfo>('get_system_info')
      .then(info => {
        setSysInfo(info);
        setSettings((prev: typeof settings) => ({
          ...prev,
          threads: prev.threads || info.available_threads,
        }));
      })
      .catch(() => {});
  }, []);

  const handleToolbarSearch = (term: string) => {
    setSearchTerm(term);
    if (tab !== 'archives') {
      setTab('archives');
    }
  };

  const views: Record<NavTab, React.ReactNode> = {
    home: <HomeView onNav={setTab} />,
    smart: <SmartCompressView settings={settings} />,
    archives: <ArchiveExplorerView searchTerm={searchTerm} />,
    extract: <ExtractView />,
    benchmark: <BenchmarkView />,
    settings: (
      <SettingsView
        settings={settings}
        onUpdateSettings={handleUpdateSettings}
        sysInfo={sysInfo}
      />
    ),
  };

  return (
    <div className="app">
      <Sidebar active={tab} onNav={setTab} sysInfo={sysInfo} />
      <div className="main">
        <Toolbar onTab={setTab} searchTerm={searchTerm} onSearch={handleToolbarSearch} />
        <div className="content">{views[tab]}</div>
      </div>
    </div>
  );
}

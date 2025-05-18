import { useState, useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

function App() {
  const [audioSessions, setAudioSessions] = useState([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);
  const containerRef = useRef(null);

  // Function to fetch audio sessions
  const fetchAudioSessions = async () => {
    try {
      setLoading(true);
      const sessions = await invoke("get_audio_sessions");
      setAudioSessions(sessions);
      setError(null);
      return sessions;
    } catch (err) {
      console.error("Error fetching audio sessions:", err);
      setError(`Failed to get audio sessions: ${err}`);
      return null;
    } finally {
      setLoading(false);
    }
  };

  // Function to handle volume change
  const handleVolumeChange = async (pid, volume) => {
    try {
      await invoke("set_volume", { pid, volume });

      // Update local state
      setAudioSessions(prevSessions =>
        prevSessions.map(session =>
          session.pid === pid ? { ...session, volume } : session
        )
      );
    } catch (err) {
      console.error(`Error setting volume for PID ${pid}:`, err);
      setError(`Failed to set volume: ${err}`);
    }
  };

  // Function to handle mute toggle
  const handleMuteToggle = async (pid, muted) => {
    try {
      await invoke("set_mute", { pid, mute: !muted });

      // Update local state
      setAudioSessions(prevSessions =>
        prevSessions.map(session =>
          session.pid === pid ? { ...session, muted: !muted } : session
        )
      );
    } catch (err) {
      console.error(`Error toggling mute for PID ${pid}:`, err);
      setError(`Failed to toggle mute: ${err}`);
    }
  };

  // Function to resize the window based on content
  const resizeWindow = () => {
    if (containerRef.current) {
      // Calculate exact height needed
      const titleBarHeight = 30; // Height of title bar
      const sessionHeight = 60; // Height per audio session
      const padding = 20; // Padding for top and bottom

      // Calculate total height based on number of sessions
      const totalHeight = titleBarHeight + (audioSessions.length * sessionHeight) + padding;

      // Set minimum height
      const minHeight = 100;
      const finalHeight = Math.max(totalHeight, minHeight);

      invoke('resize_window', { height: finalHeight })
        .catch(err => console.error('Failed to resize window:', err));
    }
  };

  // Effect to resize window when audio sessions change
  useEffect(() => {
    if (audioSessions.length > 0) {
      // Wait for the DOM to update
      setTimeout(resizeWindow, 100);
    }
  }, [audioSessions]);

  // Fetch audio sessions on component mount and periodically
  useEffect(() => {
    fetchAudioSessions();

    // Set up polling to refresh the audio sessions every 2 seconds
    const intervalId = setInterval(fetchAudioSessions, 2000);

    // Clean up interval on component unmount
    return () => clearInterval(intervalId);
  }, []);

  return (
    <main className="container" ref={containerRef}>
      <div className="title-bar">
        <div className="window-controls">
          <button
            className="window-control-button close-button"
            onClick={() => invoke('close_window')}
            title="Close"
          />
        </div>
        <h1 className="title-text">Volume Mixer</h1>
      </div>

      <div className="content-area">
        {error && <div className="error-message">{error}</div>}

        {loading && audioSessions.length === 0 ? (
          <div className="loading">Loading...</div>
        ) : audioSessions.length === 0 ? (
          <div className="no-sessions">No audio sessions</div>
        ) : (
          <div className="audio-sessions">
            {audioSessions.map((session) => (
              <div key={session.pid} className="audio-session">
                <div className="session-header">
                  {session.icon_path && (
                    <img
                      src={session.icon_path}
                      alt={`${session.name} icon`}
                      className="app-icon"
                    />
                  )}
                  <div className="session-name">{session.name}</div>
                  <button
                    className={`mute-button ${session.muted ? 'muted' : ''}`}
                    onClick={() => handleMuteToggle(session.pid, session.muted)}
                  >
                    {session.muted ? '🔇' : '🔊'}
                  </button>
                </div>

                <div className="volume-control">
                  <input
                    type="range"
                    min="0"
                    max="1"
                    step="0.01"
                    value={session.volume}
                    onChange={(e) => handleVolumeChange(session.pid, parseFloat(e.target.value))}
                    className="volume-slider"
                  />
                  <div className="volume-value">{Math.round(session.volume * 100)}%</div>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>
    </main>
  );
}

export default App;

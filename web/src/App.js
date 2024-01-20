import React, { useState, useEffect, useRef } from "react";
import init, { run } from "krusty";

function App() {
  const [state, setState] = useState({
    aperture: 1.0,
  });

  // Create a mutable reference to the state
  const stateRef = useRef(state);

  useEffect(() => {
    stateRef.current = state;
  }, [state]);

  useEffect(() => {
    loadKrust();
    document.title = "Krust GPU";
  }, []);

  // Dynamic callback function that Rust can call
  const getStateCallback = () => stateRef.current;

  async function loadKrust() {
    try {
      await init();
      await run(getStateCallback);
    } catch (error) {
      console.error("Web assembly initialization error:", error);
    }
  }

  // Callback function to change the aperture
  function changeAperture() {
    setState((prevState) => {
      return { ...prevState, aperture: prevState.aperture + 0.1 };
    });
  }

  return (
    <div className="App">
      <div id="krust-gpu" style={{ width: 1280, height: 720 }} />
      <button onClick={changeAperture}>Change Aperture</button>
    </div>
  );
}

export default App;

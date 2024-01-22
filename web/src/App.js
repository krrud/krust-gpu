import React, {useState, useEffect, useRef} from "react";
import { MantineProvider, ColorSchemeScript, DEFAULT_THEME, Text } from '@mantine/core';
import '@mantine/core/styles.css';
import init, {run} from "krusty";
import './App.css';
import { ReactComponent as Logo } from "./assets/logo_horizontal.svg";
import { AppShell, Burger, Group } from '@mantine/core';
import { useDisclosure } from '@mantine/hooks';
import { SliderInput } from "./components/SliderInput";
import { Accordion, ActionIcon, AccordionControlProps, Center } from '@mantine/core';


function App() {
  const [state, setState] = useState({
    config: {
      size: [1280, 720],
      sky_intensity: 1.0,
    },
    camera: {
      aperture: 1.0,
      fov: 50.0,
    },
    focus: true,
  });  
  const [mobileOpened, { toggle: toggleMobile }] = useDisclosure();
  const [desktopOpened, { toggle: toggleDesktop }] = useDisclosure(true);

  const stateRef = useRef(state);
  const krustRef = useRef(null);

  // Rust callback to get the current state
  const getStateCallback = () => stateRef.current;

  useEffect(() => {
    stateRef.current = state;
  }, [state]);

  useEffect(() => {
    loadKrust();
    document.title = "Krust GPU";
  }, []);

  useEffect(() => {
    const canvas = krustRef.current?.querySelector('canvas');
    canvas?.focus();
  }, [state, mobileOpened, desktopOpened]);

  useEffect(() => {
    const canvas = krustRef.current?.querySelector('canvas');
    if (canvas) {
      canvas.style.outline = 'none';
      canvas.style.borderRadius = '12px';
      canvas.focus();
    } else {
      const observer = new MutationObserver((mutationsList, observer) => {
        for(let mutation of mutationsList) {
          if (mutation.type === 'childList') {
            const canvas = krustRef.current?.querySelector('canvas');
            if (canvas) {
              canvas.style.outline = 'none';
              canvas.style.borderRadius = '12px';
              canvas.focus();
              observer.disconnect();
            }
          }
        }
      });
      observer.observe(krustRef.current, { childList: true });
    }
  }, [state]);

  async function loadKrust() {
    try {
      await init();
      await run(getStateCallback);
      setState({...state, focus: !state.focus});
    } catch (error) {
      console.error("Web assembly initialization error:", error);
    }
  }

  function changeAperture(input) {
    const value = parseFloat(Math.min(1.0, Math.max(input)).toFixed(2));
    setState({
      ...state,
      camera: {
        ...state.camera,
        aperture: value,
      },
      focus: !state.focus,
    });
  }

  function changeFov(input) {
    const value = parseFloat(Math.min(360.0, Math.max(10.0, input)).toFixed(2));
    setState({
      ...state,
      camera: {
        ...state.camera,
        fov: value,
      },
      focus: !state.focus,
    });
  }

  function changeSkyIntensity(input) {
    const value = parseFloat(Math.min(1.0, Math.max(input)).toFixed(2));
    setState({
      ...state,
      config: {
        ...state.config,
        sky_intensity: value,
      },
      focus: !state.focus,
    });
  }

  function AccordionControl(props) {
    return (
      <Center>
        <Accordion.Control {...props} />

      </Center>
    );
  }

  return (
    <>
    <ColorSchemeScript forceColorScheme="dark" />
    <MantineProvider forceColorScheme="dark">
      <AppShell
        header={{ height: 60 }}
        navbar={{
          width: 300,
          breakpoint: 'sm',
          collapsed: { mobile: !mobileOpened, desktop: !desktopOpened },
        }}
        padding="md"
        >
        <AppShell.Header>
          <Group h="100%" px="md">
            <Burger opened={mobileOpened} onClick={toggleMobile} hiddenFrom="sm" size="sm" />
            <Burger opened={desktopOpened} onClick={toggleDesktop} visibleFrom="sm" size="sm" />
            <div style={{ display: 'flex', justifyContent: 'center', alignItems: 'center', height: 0}}>
              <Logo width={180} height={160} />
            </div>
          </Group>
        </AppShell.Header>
        <AppShell.Navbar>
        <Accordion chevronPosition="left" onChange={()=>setState({...state, focus: !state.focus})}>
          <Accordion.Item value="item-1">
            <AccordionControl>Camera Settings</AccordionControl>
            <Accordion.Panel>
              <Text size="sm" mb="sm" mt={12} fw={400}>Aperture</Text>
              <SliderInput
                onChange={(val)=>changeAperture(val)}
                defaultValue={0.1}
                step={0.01}
                min={0}
                max={1}
              />

              <Text size="sm" mb="sm" mt={42} fw={400}>FoV</Text>
              <SliderInput
                onChange={(val)=>changeFov(val)}
                defaultValue={50.0}
                step={0.01}
                min={10}
                max={180}
                style={{marginBottom: 24}}
              />

            </Accordion.Panel>
          </Accordion.Item>

          <Accordion.Item value="item-2">
            <AccordionControl>Scene Settings</AccordionControl>
            <Accordion.Panel>
              <Text size="sm" mb="sm" mt={12} fw={400}>Sky Intensity</Text>
              <SliderInput
                onChange={(val)=>changeSkyIntensity(val)}
                defaultValue={1.0}
                step={0.01}
                min={0}
                max={1}
                style={{marginBottom: 24}}
              />
            </Accordion.Panel>
          </Accordion.Item>
        </Accordion>
        </AppShell.Navbar>
        <AppShell.Main
          style={{display: 'flex', justifyContent: 'center' }}
        >
          <div
            id="krust-gpu"
            style={{
              marginTop: 24,
              width: state.config.size[0],
              height: state.config.size[1],
            }}
            ref={krustRef}
            tabIndex={0}
          />
        </AppShell.Main>
      </AppShell>
    </MantineProvider>
    </>
  );
}

export default App;







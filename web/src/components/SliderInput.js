import {useState} from 'react';
import {NumberInput, Slider} from '@mantine/core';
import classes from './SliderInput.module.css';


export function SliderInput(props) {
  const [value, setValue] = useState(props.defaultValue);

  function handleChange(val) {
    setValue(val);
    props.onChange(val);
  }

  return (
    <div className={classes.wrapper} style={props.style}>
      <NumberInput
        value={value}
        onChange={(val)=>handleChange(val)}
        step={0.01}
        min={props.min ? props.min : 0}
        max={props.max ? props.max : 2}
        // hideControls
        classNames={{ input: classes.input, label: classes.label }}
      />
      <Slider
        max={props.max ? props.max : 2}
        step={0.01}
        min={props.min ? props.min : 0}
        label={null}
        value={value}
        onChange={(val)=>handleChange(val)}
        size={8}
        className={classes.slider}
        classNames={classes}
      />
    </div>
  );
}
import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

export interface ParameterConfig {
  name: string;
  node: string;     // e.g., "kick", "clap", "delay", "reverb"
  event: string;    // e.g., "set_gain", "set_attack", "set_frequency"
  min: number;
  max: number;
  step: number;
  defaultValue: number;
  unit?: string;
  formatter?: (value: number) => string;
}

export interface InstrumentConfig {
  name: string;
  color: string;
  triggerNode: string;  // e.g., "kick", "clap"
  parameters: ParameterConfig[];
}

interface AuditionerProps {
  config: InstrumentConfig;
}

export function Auditioner({ config }: AuditionerProps) {
  const [parameters, setParameters] = useState<Record<string, number>>(() => {
    const initial: Record<string, number> = {};
    config.parameters.forEach(param => {
      initial[`${param.node}.${param.event}`] = param.defaultValue;
    });
    return initial;
  });

  const updateParameter = async (param: ParameterConfig, value: number) => {
    const key = `${param.node}.${param.event}`;
    setParameters(prev => ({ ...prev, [key]: value }));
    
    try {
      await invoke("send_audio_event", {
        systemName: "auditioner",
        nodeName: param.node,
        eventName: param.event,
        parameter: value
      });
    } catch (error) {
      console.error(`Error setting ${param.name}:`, error);
    }
  };

  const triggerInstrument = async () => {
    try {
      await invoke("send_audio_event", {
        systemName: "auditioner", 
        nodeName: config.triggerNode,
        eventName: "trigger",
        parameter: 0.0
      });
    } catch (error) {
      console.error(`Error triggering ${config.name}:`, error);
    }
  };

  const formatValue = (param: ParameterConfig, value: number): string => {
    if (param.formatter) {
      return param.formatter(value);
    }
    
    if (param.unit === 'ms') {
      return `${(value * 1000).toFixed(1)}ms`;
    }
    
    if (param.unit === 'hz') {
      return `${value.toFixed(1)}Hz`;
    }
    
    if (param.unit === '%') {
      return `${(value * 100).toFixed(0)}%`;
    }
    
    return value.toFixed(3);
  };

  const getColorClasses = (color: string) => {
    switch (color) {
      case 'red':
        return {
          title: 'text-red-400',
          button: 'bg-red-600 hover:bg-red-700'
        };
      case 'blue':
        return {
          title: 'text-blue-400',
          button: 'bg-blue-600 hover:bg-blue-700'
        };
      case 'green':
        return {
          title: 'text-green-400',
          button: 'bg-green-600 hover:bg-green-700'
        };
      case 'purple':
        return {
          title: 'text-purple-400',
          button: 'bg-purple-600 hover:bg-purple-700'
        };
      case 'yellow':
        return {
          title: 'text-yellow-400',
          button: 'bg-yellow-600 hover:bg-yellow-700'
        };
      default:
        return {
          title: 'text-gray-400',
          button: 'bg-gray-600 hover:bg-gray-700'
        };
    }
  };

  const colorClasses = getColorClasses(config.color);

  return (
    <div className="bg-gray-800 rounded-xl p-6 border border-gray-700">
      <div className="flex justify-between items-center mb-6">
        <h2 className={`text-2xl font-bold ${colorClasses.title}`}>
          {config.name}
        </h2>
        <button
          onClick={triggerInstrument}
          className={`${colorClasses.button} text-white font-bold py-3 px-6 rounded-lg transition-all transform hover:scale-105 shadow-lg`}
        >
          ▶ Trigger
        </button>
      </div>

      <div className="space-y-6">
        {config.parameters.map((param) => {
          const key = `${param.node}.${param.event}`;
          return (
            <div key={key}>
              <label className="block text-sm font-bold mb-2">
                {param.name}: {formatValue(param, parameters[key])}
              </label>
              <input
                type="range"
                min={param.min}
                max={param.max}
                step={param.step}
                value={parameters[key]}
                onChange={(e) => updateParameter(param, parseFloat(e.target.value))}
                className="w-full h-2 bg-gray-700 rounded-lg appearance-none cursor-pointer slider"
              />
            </div>
          );
        })}
      </div>
    </div>
  );
}
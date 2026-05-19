interface RadioOption<T extends string> {
  value: T;
  label: string;
}

interface RadioControlProps<T extends string> {
  value: T;
  options: ReadonlyArray<RadioOption<T>>;
  onChange: (value: T) => void;
}

export function RadioControl<T extends string>({ value, options, onChange }: RadioControlProps<T>) {
  return (
    <div className="sub-control" role="radiogroup">
      {options.map((opt) => (
        <button
          type="button"
          key={opt.value}
          className="sub-radio"
          role="radio"
          aria-checked={value === opt.value}
          onClick={() => onChange(opt.value)}
        >
          <span className="sub-radio-dot" />
          {opt.label}
        </button>
      ))}
    </div>
  );
}

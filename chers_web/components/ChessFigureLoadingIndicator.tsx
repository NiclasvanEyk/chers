interface Props {
  message?: string;
  fullscreen?: boolean;
}

export function ChessFigureLoadingIndicator({ message, fullscreen }: Props) {
  function Indicator() {
    return (
      <div className="flex flex-col gap-4 items-center">
        <img
          src="/images/pieces/White_Unicorn.svg"
          className="h-10 w-10 animate-spin"
        />
        {message ? <span>{message}</span> : null}
      </div>
    );
  }

  if (!fullscreen) {
    return <Indicator />;
  }

  return (
    <div className="flex min-h-screen flex-col items-center justify-center">
      <Indicator />
    </div>
  );
}

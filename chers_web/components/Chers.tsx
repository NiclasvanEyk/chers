"use client";

import { useEffect, useState } from "react";
import init from "@/lib/chers/chers";
import { Board } from "./Board";

export function LoadingIndicator(props: any) {
  return <img src="/pieces/White_Unicorn.svg" {...props} />;
}

export default function Chers() {
  const [loading, setLoading] = useState(true);

  useEffect(function () {
    init().then(() => setLoading(false));
  }, []);

  if (loading) {
    return (
      <div className="h-screen w-screen flex justify-center items-center">
        <LoadingIndicator className="h-10 w-10 animate-spin" />
      </div>
    );
  }

  return <Board />;
}

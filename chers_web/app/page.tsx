"use client";

import Chers from "@/components/Chers";
import { ReactNode, useEffect, useState } from "react";
import init from "@/generated/chers/chers";

export function LoadingIndicator(props: any) {
  return <img src="/images/pieces/White_Unicorn.svg" {...props} />;
}

function Container(props: { children: ReactNode }) {
  return (
    <main className="flex min-h-screen flex-col items-center justify-center">
      {props.children}
    </main>
  );
}

export default function Home() {
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    init().then(() => setLoading(false));
  }, []);

  if (loading) {
    return (
      <Container>
        <LoadingIndicator />
      </Container>
    );
  }

  return (
    <Container>
      <Chers />
    </Container>
  );
}

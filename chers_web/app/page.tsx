import Image from "next/image";
import Chers from "@/components/Chers";

export default function Home() {
  return (
    <main className="flex min-h-screen flex-col items-center justify-between p-24">
      <Chers />
    </main>
  );
}

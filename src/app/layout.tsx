import type { Metadata, Viewport } from "next";
import { Inter } from "next/font/google";
import ErrorBoundary from "@/components/debug/ErrorBoundary";
import "./globals.css";

const inter = Inter({
  variable: "--font-inter",
  subsets: ["latin"],
  display: "swap",
  preload: true,
});

// Mobile-first viewport — prevents browser zoom, covers notch areas
export const viewport: Viewport = {
  themeColor: "#000000",
  width: "device-width",
  initialScale: 1,
  maximumScale: 1,
  userScalable: false,
  viewportFit: "cover",
};

export const metadata: Metadata = {
  title: "BlackHole",
  description:
    "Real-time interactive black hole simulation. Tap the center to fall in.",
  appleWebApp: {
    capable: true,
    title: "BlackHole",
    statusBarStyle: "black-translucent",
  },
  other: {
    "mobile-web-app-capable": "yes",
    "msapplication-TileColor": "#000000",
    "msapplication-tap-highlight": "no",
  },
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en" suppressHydrationWarning>
      <body
        className={`${inter.variable} antialiased bg-black`}
        suppressHydrationWarning
        style={{ margin: 0, padding: 0, overflow: "hidden" }}
      >
        <ErrorBoundary>{children}</ErrorBoundary>
      </body>
    </html>
  );
}

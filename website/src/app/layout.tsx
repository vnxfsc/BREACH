import type { Metadata } from "next";
import "./globals.css";
import BackgroundEffects from "@/components/BackgroundEffects";

const siteUrl = process.env.NEXT_PUBLIC_SITE_URL || "https://breach-jade.vercel.app";

export const metadata: Metadata = {
  metadataBase: new URL(siteUrl),
  title: {
    default: "BREACH - Hunt. Capture. Dominate.",
    template: "%s | BREACH",
  },
  description: "A Solana-powered AR monster hunting game. Hunt massive Titans emerging from dimensional rifts and build your army.",
  keywords: [
    "BREACH",
    "Web3 Game",
    "Solana",
    "NFT Game",
    "AR Game",
    "Monster Hunting",
    "Titan",
    "Blockchain Gaming",
    "Play to Earn",
    "Crypto Game",
  ],
  authors: [{ name: "BREACH Team" }],
  creator: "BREACH",
  publisher: "BREACH",
  robots: {
    index: true,
    follow: true,
    googleBot: {
      index: true,
      follow: true,
      "max-video-preview": -1,
      "max-image-preview": "large",
      "max-snippet": -1,
    },
  },
  openGraph: {
    type: "website",
    locale: "en_US",
    url: siteUrl,
    siteName: "BREACH",
    title: "BREACH - Hunt. Capture. Dominate.",
    description: "A Solana-powered AR monster hunting game. Hunt massive Titans emerging from dimensional rifts and build your army.",
    images: [
      {
        url: "/og-image.svg",
        width: 1200,
        height: 630,
        alt: "BREACH - Solana AR Monster Hunting Game",
      },
    ],
  },
  twitter: {
    card: "summary_large_image",
    site: "@BreachGame",
    creator: "@BreachGame",
    title: "BREACH - Hunt. Capture. Dominate.",
    description: "A Solana-powered AR monster hunting game. Hunt massive Titans emerging from dimensional rifts and build your army.",
    images: ["/og-image.svg"],
  },
  icons: {
    icon: "/favicon.ico",
    apple: "/apple-touch-icon.png",
  },
  manifest: "/manifest.json",
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en" className="dark">
      <body className="antialiased min-h-screen bg-[var(--color-bg-dark)] overflow-x-hidden">
        <BackgroundEffects />
        <div className="relative z-10">
          {children}
        </div>
      </body>
    </html>
  );
}

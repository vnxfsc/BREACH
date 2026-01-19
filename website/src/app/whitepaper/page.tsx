import Navbar from "@/components/Navbar";
import Footer from "@/components/Footer";
import PageHeader from "@/components/PageHeader";
import { Metadata } from "next";
import { FileText, Download } from "lucide-react";

export const metadata: Metadata = {
  title: "Whitepaper - BREACH",
  description: "BREACH technical whitepaper detailing game mechanics, tokenomics, and blockchain architecture.",
};

const sections = [
  {
    number: "01",
    title: "Executive Summary",
    content: `BREACH is a location-based augmented reality game built on Solana that combines 
    monster collecting, strategic battles, and true digital ownership. Players hunt and capture 
    massive creatures called Titans emerging from dimensional rifts, building armies and competing 
    globally. Every Titan is a unique NFT with verifiable on-chain attributes, enabling a player-owned 
    economy where assets retain real value.`,
  },
  {
    number: "02",
    title: "Game Vision",
    content: `Our vision is to create the definitive Web3 monster hunting experience. By leveraging 
    AR technology and blockchain, we deliver gameplay that rewards skill and time investment with 
    true ownership. Unlike traditional games where digital items are locked to accounts, BREACH 
    players truly own their Titans — they can be traded, sold, or held as collectibles independent 
    of the game platform.`,
  },
  {
    number: "03",
    title: "Titan System",
    subsections: [
      {
        subtitle: "Classification",
        text: `Titans are classified into five threat classes (I-V) representing power and rarity. 
        Class I "Pioneer" Titans are common (60% spawn rate), while Class V "Apex" Titans are legendary 
        (1% spawn rate) requiring coordinated guild efforts to capture.`,
      },
      {
        subtitle: "Elemental Types",
        text: `Six elemental types exist in a circular effectiveness relationship: Abyssal → Volcanic 
        → Storm → Void → Parasitic → Ossified → Abyssal. Type matchups add strategic depth to team 
        building and battles.`,
      },
      {
        subtitle: "Attributes & Genes",
        text: `Each Titan has four visible attributes (Power, Fortitude, Velocity, Resonance) and a 
        hidden 6-byte gene sequence determining growth potential. Genes influence stat scaling, ability 
        unlocks, and fusion outcomes. S-rank genes are highly sought after.`,
      },
    ],
  },
  {
    number: "04",
    title: "Neural Link Capture",
    content: `The Neural Link is our signature capture mechanic — a rhythm-based minigame where 
    players synchronize neural patterns with target Titans. Success rate depends on accuracy and 
    timing. Higher class Titans require greater precision, with Class V demanding near-perfect 
    execution. This skill-based system rewards dedicated players over pay-to-win mechanics.`,
  },
  {
    number: "05",
    title: "Battle System",
    subsections: [
      {
        subtitle: "PvE",
        text: `Auto-battle system for hunting wild Titans and completing missions. Players build 
        teams of 3 Titans optimizing type coverage and synergies. Daily rewards scale with performance.`,
      },
      {
        subtitle: "PvP",
        text: `Turn-based strategic battles in ranked Arena mode. Players take turns commanding 
        abilities, managing resources, and predicting opponent moves. Seasonal rankings award 
        exclusive Titans and tokens.`,
      },
      {
        subtitle: "World Bosses",
        text: `Massive limited-time events where Class V+ Titans threaten regions. Guilds must 
        coordinate attacks, with contribution-based rewards including rare genes and unique items.`,
      },
    ],
  },
  {
    number: "06",
    title: "Tokenomics ($BREACH)",
    subsections: [
      {
        subtitle: "Token Utility",
        text: `$BREACH is the native SPL token powering the economy: capture costs, upgrades, 
        fusion fees, marketplace transactions, and governance voting. It creates sustainable 
        demand through core gameplay loops.`,
      },
      {
        subtitle: "Distribution",
        text: `Total supply: 1,000,000,000 tokens. Allocation: Game Rewards (40%), Team (20%), 
        Ecosystem (15%), Investors (15%), Liquidity (10%). Team tokens vest over 4 years with 
        1-year cliff.`,
      },
      {
        subtitle: "Earning Mechanisms",
        text: `Players earn through PvE victories, PvP rankings, quest completion, staking 
        rewards, and marketplace trading fees. Deflationary mechanics include capture burns 
        and fusion costs.`,
      },
    ],
  },
  {
    number: "07",
    title: "Technical Architecture",
    subsections: [
      {
        subtitle: "Blockchain Layer",
        text: `Built on Solana for sub-second finality and minimal fees. Two custom programs: 
        Titan NFT Program (minting, attributes, evolution) and Game Logic Program (capture validation, 
        battle records). $BREACH token uses standard SPL Token for full DEX compatibility (Raydium, 
        Orca, Jupiter). NFT trading via Magic Eden/Tensor.`,
      },
      {
        subtitle: "Backend Services",
        text: `High-performance Rust backend using Axum/Tokio for async processing. PostgreSQL 
        with PostGIS for geospatial queries, Redis for real-time state, and custom battle engine 
        for deterministic combat simulation.`,
      },
      {
        subtitle: "Client Applications",
        text: `Cross-platform Flutter application with native AR integration (ARKit/ARCore). 
        Optimized for mobile performance with offline caching and background sync.`,
      },
    ],
  },
  {
    number: "08",
    title: "Roadmap",
    content: `Q1 2026: Core development, smart contracts, website launch. Q2 2026: Closed alpha, 
    Neural Link system, basic PvE. Q3 2026: Public beta, $BREACH launch, marketplace. Q4 2026: 
    Global launch, PvP ranked, World Bosses. 2027+: Cross-chain expansion, mobile esports, 
    metaverse integration.`,
  },
  {
    number: "09",
    title: "Conclusion",
    content: `BREACH represents the next evolution of monster collecting games — where players 
    truly own their digital assets, skill determines success, and a global community shares in 
    the adventure. By combining proven gameplay mechanics with blockchain technology, we're 
    building a sustainable ecosystem that rewards players for their time and dedication. 
    The hunt begins now.`,
  },
];

export default function WhitepaperPage() {
  return (
    <main className="min-h-screen bg-[var(--color-bg-dark)]">
      <Navbar />
      <PageHeader 
        title="Whitepaper" 
        subtitle="Technical documentation for BREACH ecosystem"
      />
      
      <div className="max-w-4xl mx-auto px-4 sm:px-6 lg:px-8 pb-24">
        {/* Download Button */}
        <div className="flex justify-center mb-12">
          <a
            href="#"
            className="inline-flex items-center gap-2 px-6 py-3 bg-[var(--color-bg-card)] border border-white/10 hover:border-[var(--color-primary)]/50 rounded-lg transition-all duration-200 group"
          >
            <FileText className="w-5 h-5 text-[var(--color-primary)]" />
            <span>Download PDF</span>
            <Download className="w-4 h-4 text-[var(--color-text-muted)] group-hover:text-[var(--color-primary)] transition-colors" />
          </a>
        </div>

        {/* Content */}
        <div className="space-y-16">
          {sections.map((section) => (
            <section key={section.number} className="scroll-mt-24" id={`section-${section.number}`}>
              <div className="flex items-start gap-4 mb-6">
                <span className="text-4xl font-black text-gradient opacity-50">{section.number}</span>
                <h2 className="text-2xl sm:text-3xl font-bold pt-2">{section.title}</h2>
              </div>
              
              {section.content && (
                <p className="text-[var(--color-text-secondary)] leading-relaxed mb-6">
                  {section.content}
                </p>
              )}
              
              {section.subsections && (
                <div className="space-y-6 pl-4 border-l-2 border-[var(--color-primary)]/20">
                  {section.subsections.map((sub) => (
                    <div key={sub.subtitle}>
                      <h3 className="text-lg font-semibold mb-2 text-[var(--color-primary)]">
                        {sub.subtitle}
                      </h3>
                      <p className="text-[var(--color-text-secondary)] text-sm leading-relaxed">
                        {sub.text}
                      </p>
                    </div>
                  ))}
                </div>
              )}
            </section>
          ))}
        </div>

        {/* Version Info */}
        <div className="mt-16 pt-8 border-t border-white/5 text-center text-[var(--color-text-muted)] text-sm">
          <p>Whitepaper v1.0 — Last updated January 2026</p>
          <p className="mt-1">This document is subject to change as development progresses.</p>
        </div>
      </div>

      <Footer />
    </main>
  );
}

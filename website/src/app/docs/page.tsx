import Navbar from "@/components/Navbar";
import Footer from "@/components/Footer";
import PageHeader from "@/components/PageHeader";
import { Metadata } from "next";

export const metadata: Metadata = {
  title: "Documentation - BREACH",
  description: "Learn how to play BREACH, understand Titan mechanics, and master the Neural Link system.",
};

const docSections = [
  {
    title: "Getting Started",
    items: [
      {
        title: "What is BREACH?",
        content: "BREACH is a Solana-powered AR monster hunting game where players capture and battle massive Titans emerging from dimensional rifts. As a Linker, you'll use the Neural Link system to tame these colossal beings.",
      },
      {
        title: "Creating Your Account",
        content: "Connect your Solana wallet (Phantom, Solflare, etc.) to create your Linker profile. Your wallet stores your Titans as NFTs and your $BREACH tokens.",
      },
      {
        title: "Your First Hunt",
        content: "Open the AR scanner to detect nearby Breach signals. Approach the rift, initiate the Neural Link capture sequence, and sync your neural patterns to capture your first Titan.",
      },
    ],
  },
  {
    title: "Titan Mechanics",
    items: [
      {
        title: "Threat Classes (I-V)",
        content: "Titans are classified by threat level from I (Pioneer) to V (Apex). Higher classes are rarer, more powerful, and require advanced Linker skills to capture.",
      },
      {
        title: "Elemental Types",
        content: "Six elemental types exist: Abyssal, Volcanic, Storm, Void, Parasitic, and Ossified. Each type has strengths and weaknesses against others in a circular relationship.",
      },
      {
        title: "Gene System",
        content: "Every Titan has a unique 6-byte gene sequence determining hidden potential. Genes affect stat growth, ability unlocks, and fusion outcomes. Hunt for S-rank genes!",
      },
      {
        title: "Attributes",
        content: "Four visible stats: Power (attack damage), Fortitude (defense), Velocity (speed/initiative), and Resonance (special ability strength). These determine battle performance.",
      },
    ],
  },
  {
    title: "Neural Link System",
    items: [
      {
        title: "How Capture Works",
        content: "The Neural Link is a rhythm-based minigame. Match the neural pattern sequence displayed on screen. Higher accuracy = higher capture rate. Class V Titans require perfect sync!",
      },
      {
        title: "Link Strength",
        content: "Your bond with captured Titans grows over time. Higher link strength unlocks abilities, improves battle performance, and enables fusion.",
      },
      {
        title: "Failed Links",
        content: "Failed capture attempts don't lose the Titan — it escapes but may reappear. However, you consume capture resources ($BREACH) on each attempt.",
      },
    ],
  },
  {
    title: "Battle System",
    items: [
      {
        title: "PvE Battles",
        content: "Auto-battle system for hunting wild Titans and completing daily missions. Build a team of 3 Titans with complementary types and abilities.",
      },
      {
        title: "PvP Arena",
        content: "Turn-based strategic battles against other Linkers. Climb the ranked ladder, earn seasonal rewards, and prove your mastery.",
      },
      {
        title: "World Bosses",
        content: "Massive Class V+ Titans appear at special events. Guilds must cooperate to take them down. Rewards include rare genes and exclusive items.",
      },
    ],
  },
  {
    title: "Economy",
    items: [
      {
        title: "$BREACH Token",
        content: "The native utility token used for captures, upgrades, fusion, marketplace trades, and governance voting. Earn through battles, quests, and staking.",
      },
      {
        title: "Marketplace",
        content: "Trade Titans with other players. Each Titan is a unique NFT with verifiable on-chain attributes. List, bid, and negotiate freely.",
      },
      {
        title: "Fusion",
        content: "Combine two Titans to create a new one inheriting traits from both parents. Gene combinations can produce rare mutations!",
      },
    ],
  },
];

export default function DocsPage() {
  return (
    <main className="min-h-screen bg-[var(--color-bg-dark)]">
      <Navbar />
      <PageHeader 
        title="Documentation" 
        subtitle="Everything you need to know about BREACH"
      />
      
      <div className="max-w-4xl mx-auto px-4 sm:px-6 lg:px-8 pb-24">
        <div className="space-y-12">
          {docSections.map((section, sectionIndex) => (
            <section key={section.title} className="scroll-mt-24" id={section.title.toLowerCase().replace(/\s+/g, '-')}>
              <h2 className="text-2xl font-bold mb-6 text-gradient">{section.title}</h2>
              <div className="space-y-4">
                {section.items.map((item, itemIndex) => (
                  <div
                    key={item.title}
                    className="bg-[var(--color-bg-card)] border border-white/5 rounded-xl p-6 hover:border-white/10 transition-colors"
                  >
                    <h3 className="text-lg font-semibold mb-2">{item.title}</h3>
                    <p className="text-[var(--color-text-secondary)] text-sm leading-relaxed">
                      {item.content}
                    </p>
                  </div>
                ))}
              </div>
            </section>
          ))}
        </div>

        {/* Table of Contents Sidebar */}
        <aside className="hidden xl:block fixed right-8 top-32 w-56">
          <h4 className="text-sm font-semibold mb-4 text-[var(--color-text-muted)]">On This Page</h4>
          <nav className="space-y-2">
            {docSections.map((section) => (
              <a
                key={section.title}
                href={`#${section.title.toLowerCase().replace(/\s+/g, '-')}`}
                className="block text-sm text-[var(--color-text-secondary)] hover:text-[var(--color-primary)] transition-colors"
              >
                {section.title}
              </a>
            ))}
          </nav>
        </aside>
      </div>

      <Footer />
    </main>
  );
}

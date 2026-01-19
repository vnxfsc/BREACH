"use client";

import { useState } from "react";
import Navbar from "@/components/Navbar";
import Footer from "@/components/Footer";
import PageHeader from "@/components/PageHeader";
import { motion, AnimatePresence } from "framer-motion";
import { ChevronDown } from "lucide-react";

const faqCategories = [
  {
    category: "General",
    questions: [
      {
        q: "What is BREACH?",
        a: "BREACH is a Solana-powered AR monster hunting game. Players hunt and capture massive Titans emerging from dimensional rifts, build teams, battle other players, and trade Titans as NFTs.",
      },
      {
        q: "Is BREACH free to play?",
        a: "Yes! BREACH is free to download and play. You can hunt and capture Titans without spending money. However, premium features like faster captures and cosmetics may require $BREACH tokens.",
      },
      {
        q: "What platforms is BREACH available on?",
        a: "BREACH will launch on iOS and Android devices. AR features require a device with ARKit (iOS) or ARCore (Android) support.",
      },
      {
        q: "When does BREACH launch?",
        a: "We're targeting Q4 2026 for global launch. Join our waitlist to get early access during beta phases!",
      },
    ],
  },
  {
    category: "Web3 & Blockchain",
    questions: [
      {
        q: "Do I need a crypto wallet to play?",
        a: "Yes, you'll need a Solana wallet (like Phantom or Solflare) to store your Titans and $BREACH tokens. Don't worry — we provide in-app wallet creation for newcomers.",
      },
      {
        q: "Are Titans really NFTs?",
        a: "Yes! Every Titan is a unique NFT stored on the Solana blockchain. You truly own your Titans and can trade them freely on any Solana NFT marketplace.",
      },
      {
        q: "What is $BREACH token used for?",
        a: "$BREACH is the utility token for captures, upgrades, fusion, marketplace fees, and governance voting. You can earn it through gameplay or purchase on exchanges.",
      },
      {
        q: "Are there gas fees?",
        a: "Solana has extremely low transaction fees (fractions of a cent). We also batch transactions and use compressed NFTs to minimize costs.",
      },
      {
        q: "Can I play without understanding crypto?",
        a: "Absolutely! The game handles all blockchain complexity behind the scenes. You just play the game — capturing, battling, and trading feels like any mobile game.",
      },
    ],
  },
  {
    category: "Gameplay",
    questions: [
      {
        q: "How do I capture Titans?",
        a: "Use the AR scanner to find Breach signals nearby. Approach the rift, initiate Neural Link, and complete the rhythm-based capture minigame. Higher accuracy = better capture chance!",
      },
      {
        q: "What are Titan Classes?",
        a: "Classes (I-V) represent power and rarity. Class I (Pioneer) are common, while Class V (Apex) are legendary and require guild cooperation to capture.",
      },
      {
        q: "How does the type system work?",
        a: "Six types in a circular effectiveness chain: Abyssal → Volcanic → Storm → Void → Parasitic → Ossified → Abyssal. Use type advantages in battle!",
      },
      {
        q: "Can I battle other players?",
        a: "Yes! PvP Arena features turn-based battles against other Linkers. Climb the ranked ladder and earn seasonal rewards.",
      },
      {
        q: "What is Titan Fusion?",
        a: "Combine two Titans to create a new one with inherited traits. Gene combinations can produce rare mutations and unique abilities!",
      },
    ],
  },
  {
    category: "Technical",
    questions: [
      {
        q: "What are the device requirements?",
        a: "iOS 14+ with ARKit support, or Android 8+ with ARCore support. 4GB RAM minimum recommended for smooth AR performance.",
      },
      {
        q: "Does the game require internet?",
        a: "Yes, an internet connection is required to sync with the blockchain and access real-time Breach signals. Offline mode for viewing your collection is planned.",
      },
      {
        q: "How is location data used?",
        a: "Location determines Breach spawn points and local events. We only process location when the app is active and never sell your data. See our Privacy Policy for details.",
      },
      {
        q: "Is my wallet secure?",
        a: "Your wallet private keys are stored locally on your device and never transmitted to our servers. We recommend using hardware wallet integration for large holdings.",
      },
    ],
  },
];

function FAQItem({ question, answer }: { question: string; answer: string }) {
  const [isOpen, setIsOpen] = useState(false);

  return (
    <div className="bg-[var(--color-bg-card)] border border-white/5 rounded-xl overflow-hidden">
      <button
        onClick={() => setIsOpen(!isOpen)}
        className="w-full px-6 py-4 flex items-center justify-between text-left hover:bg-white/5 transition-colors"
      >
        <span className="font-medium pr-4">{question}</span>
        <ChevronDown
          className={`w-5 h-5 text-[var(--color-text-muted)] transition-transform duration-200 flex-shrink-0 ${
            isOpen ? "rotate-180" : ""
          }`}
        />
      </button>
      <AnimatePresence>
        {isOpen && (
          <motion.div
            initial={{ height: 0, opacity: 0 }}
            animate={{ height: "auto", opacity: 1 }}
            exit={{ height: 0, opacity: 0 }}
            transition={{ duration: 0.2 }}
          >
            <div className="px-6 pb-4 text-[var(--color-text-secondary)] text-sm leading-relaxed border-t border-white/5 pt-4">
              {answer}
            </div>
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
}

export default function FAQPage() {
  return (
    <main className="min-h-screen bg-[var(--color-bg-dark)]">
      <Navbar />
      <PageHeader 
        title="FAQ" 
        subtitle="Frequently asked questions about BREACH"
      />
      
      <div className="max-w-3xl mx-auto px-4 sm:px-6 lg:px-8 pb-24">
        <div className="space-y-12">
          {faqCategories.map((category) => (
            <section key={category.category}>
              <h2 className="text-xl font-bold mb-4 text-gradient">{category.category}</h2>
              <div className="space-y-3">
                {category.questions.map((item) => (
                  <FAQItem key={item.q} question={item.q} answer={item.a} />
                ))}
              </div>
            </section>
          ))}
        </div>

        {/* Contact CTA */}
        <div className="mt-16 text-center">
          <p className="text-[var(--color-text-secondary)] mb-4">
            Still have questions?
          </p>
          <a
            href="https://github.com/vnxfsc/BREACH/discussions"
            target="_blank"
            rel="noopener noreferrer"
            className="inline-flex items-center px-6 py-3 bg-[var(--color-primary)] hover:bg-[var(--color-primary-dark)] text-black font-semibold rounded-lg transition-all duration-200"
          >
            Ask on GitHub
          </a>
        </div>
      </div>

      <Footer />
    </main>
  );
}

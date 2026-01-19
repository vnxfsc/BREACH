import Navbar from "@/components/Navbar";
import Footer from "@/components/Footer";
import PageHeader from "@/components/PageHeader";
import { Metadata } from "next";

export const metadata: Metadata = {
  title: "Terms of Service - BREACH",
  description: "BREACH terms of service governing use of our game and platform.",
};

const sections = [
  {
    title: "1. Acceptance of Terms",
    content: `By accessing or using BREACH ("the Game", "Service"), you agree to be bound by 
    these Terms of Service ("Terms"). If you do not agree to these Terms, do not use the 
    Service. We reserve the right to modify these Terms at any time. Continued use after 
    changes constitutes acceptance.`,
  },
  {
    title: "2. Eligibility",
    content: `You must be at least 13 years old (or the age of digital consent in your 
    jurisdiction) to use the Service. If you are under 18, you must have parental or 
    guardian consent. By using the Service, you represent that you meet these requirements.`,
  },
  {
    title: "3. Account Registration",
    subsections: [
      {
        subtitle: "3.1 Wallet Connection",
        text: `The Service requires connection to a Solana-compatible cryptocurrency wallet. 
        You are solely responsible for maintaining the security of your wallet and private 
        keys. We have no access to your private keys and cannot recover lost wallets.`,
      },
      {
        subtitle: "3.2 Account Security",
        text: `You are responsible for all activity on your account. Notify us immediately 
        of any unauthorized access. We are not liable for losses due to compromised accounts 
        or wallets.`,
      },
      {
        subtitle: "3.3 One Account Policy",
        text: `Each user may only maintain one active account. Creating multiple accounts to 
        circumvent restrictions or gain unfair advantages is prohibited and may result in 
        permanent bans.`,
      },
    ],
  },
  {
    title: "4. Digital Assets (NFTs & Tokens)",
    subsections: [
      {
        subtitle: "4.1 Ownership",
        text: `Titans are non-fungible tokens (NFTs) on the Solana blockchain. When you 
        capture or purchase a Titan, you own the NFT. However, we retain intellectual 
        property rights to the underlying artwork, names, and game mechanics.`,
      },
      {
        subtitle: "4.2 License",
        text: `You receive a limited, non-exclusive license to use, display, and trade your 
        owned NFTs within the Service and on compatible marketplaces. This license does not 
        include rights to commercialize, modify, or create derivative works.`,
      },
      {
        subtitle: "4.3 $BREACH Token",
        text: `$BREACH is an in-game utility token. It is NOT an investment, security, or 
        currency. Token value may fluctuate and we make no guarantees regarding future value. 
        You are responsible for complying with tax obligations in your jurisdiction.`,
      },
      {
        subtitle: "4.4 Risks",
        text: `Blockchain transactions are irreversible. Lost tokens or NFTs cannot be 
        recovered. Smart contract bugs, network failures, or market volatility may affect 
        your assets. You accept these risks by using the Service.`,
      },
    ],
  },
  {
    title: "5. Prohibited Conduct",
    content: `You agree NOT to:
    
    • Cheat, hack, or exploit bugs for unfair advantage
    • Use bots, scripts, or automation tools
    • Manipulate markets or engage in wash trading
    • Harass, threaten, or abuse other users
    • Impersonate others or provide false information
    • Reverse engineer or decompile the Service
    • Interfere with servers or network infrastructure
    • Violate any applicable laws or regulations
    • Promote illegal activities or harmful content
    
    Violations may result in account suspension, asset forfeiture, and legal action.`,
  },
  {
    title: "6. Gameplay & Virtual Items",
    subsections: [
      {
        subtitle: "6.1 Game Balance",
        text: `We reserve the right to modify game mechanics, Titan statistics, token 
        economics, and other gameplay elements at any time to maintain balance and fairness. 
        Such changes are not grounds for refunds.`,
      },
      {
        subtitle: "6.2 Service Availability",
        text: `We strive for continuous availability but do not guarantee uninterrupted 
        service. Maintenance, updates, or unforeseen issues may cause temporary downtime. 
        We are not liable for losses during outages.`,
      },
      {
        subtitle: "6.3 Virtual Economy",
        text: `The in-game economy is subject to change. We may implement inflation controls, 
        burning mechanisms, or other adjustments. Real-money value of virtual items is not 
        guaranteed.`,
      },
    ],
  },
  {
    title: "7. User Content",
    content: `Any content you submit (usernames, messages, feedback) grants us a 
    non-exclusive, royalty-free, worldwide license to use, display, and distribute such 
    content. You represent that you own or have rights to submitted content and that it 
    does not violate third-party rights.`,
  },
  {
    title: "8. Intellectual Property",
    content: `All content in the Service — including but not limited to graphics, code, 
    Titan designs, music, and trademarks — is owned by us or our licensors. You may not 
    copy, modify, distribute, or create derivative works without explicit permission. 
    NFT ownership does not transfer intellectual property rights.`,
  },
  {
    title: "9. Disclaimers",
    content: `THE SERVICE IS PROVIDED "AS IS" WITHOUT WARRANTIES OF ANY KIND, EXPRESS OR 
    IMPLIED. WE DISCLAIM ALL WARRANTIES INCLUDING MERCHANTABILITY, FITNESS FOR A PARTICULAR 
    PURPOSE, AND NON-INFRINGEMENT. WE DO NOT WARRANT THAT THE SERVICE WILL BE ERROR-FREE, 
    SECURE, OR UNINTERRUPTED.`,
  },
  {
    title: "10. Limitation of Liability",
    content: `TO THE MAXIMUM EXTENT PERMITTED BY LAW, WE SHALL NOT BE LIABLE FOR ANY 
    INDIRECT, INCIDENTAL, SPECIAL, CONSEQUENTIAL, OR PUNITIVE DAMAGES, INCLUDING LOSS OF 
    PROFITS, DATA, OR DIGITAL ASSETS, ARISING FROM YOUR USE OF THE SERVICE. OUR TOTAL 
    LIABILITY SHALL NOT EXCEED THE AMOUNT YOU PAID US IN THE 12 MONTHS PRECEDING THE CLAIM.`,
  },
  {
    title: "11. Indemnification",
    content: `You agree to indemnify and hold harmless BREACH, its affiliates, officers, 
    employees, and agents from any claims, damages, losses, or expenses arising from your 
    use of the Service, violation of these Terms, or infringement of third-party rights.`,
  },
  {
    title: "12. Dispute Resolution",
    subsections: [
      {
        subtitle: "12.1 Governing Law",
        text: `These Terms are governed by the laws of [Jurisdiction], without regard to 
        conflict of law principles.`,
      },
      {
        subtitle: "12.2 Arbitration",
        text: `Any disputes shall be resolved through binding arbitration rather than court 
        litigation. You waive the right to participate in class actions. Small claims court 
        remains available for qualifying disputes.`,
      },
    ],
  },
  {
    title: "13. Termination",
    content: `We may suspend or terminate your access to the Service at any time, with or 
    without cause or notice. Upon termination, your license to use the Service ends. 
    Your owned NFTs remain in your wallet, but in-game functionality may be disabled.`,
  },
  {
    title: "14. Severability",
    content: `If any provision of these Terms is found invalid or unenforceable, the 
    remaining provisions shall continue in full force. The invalid provision shall be 
    modified to the minimum extent necessary to make it enforceable.`,
  },
  {
    title: "15. Contact",
    content: `For questions about these Terms, contact us at:
    
    GitHub: github.com/vnxfsc/BREACH/issues
    Website: breach-jade.vercel.app`,
  },
];

export default function TermsPage() {
  return (
    <main className="min-h-screen bg-[var(--color-bg-dark)]">
      <Navbar />
      <PageHeader 
        title="Terms of Service" 
        subtitle="Rules governing use of BREACH"
      />
      
      <div className="max-w-3xl mx-auto px-4 sm:px-6 lg:px-8 pb-24">
        <div className="bg-[var(--color-bg-card)] border border-white/5 rounded-xl p-6 md:p-8 mb-8">
          <p className="text-[var(--color-text-muted)] text-sm">
            Last updated: January 2026
          </p>
          <p className="text-[var(--color-accent)] text-sm mt-2">
            Please read these terms carefully before using BREACH.
          </p>
        </div>

        <div className="space-y-8">
          {sections.map((section) => (
            <section key={section.title}>
              <h2 className="text-xl font-bold mb-4">{section.title}</h2>
              
              {section.content && (
                <p className="text-[var(--color-text-secondary)] text-sm leading-relaxed whitespace-pre-line">
                  {section.content}
                </p>
              )}
              
              {section.subsections && (
                <div className="space-y-4">
                  {section.subsections.map((sub) => (
                    <div key={sub.subtitle}>
                      <h3 className="text-base font-semibold mb-2 text-[var(--color-text-primary)]">
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

        {/* Acceptance Notice */}
        <div className="mt-12 p-6 bg-[var(--color-primary)]/10 border border-[var(--color-primary)]/20 rounded-xl">
          <p className="text-sm text-center">
            By using BREACH, you acknowledge that you have read, understood, and agree to be 
            bound by these Terms of Service.
          </p>
        </div>
      </div>

      <Footer />
    </main>
  );
}

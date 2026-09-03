import { Crown, Mail, Phone, MapPin } from 'lucide-react'

const Footer = () => {
  return (
    <footer className="bg-luxury-midnight-black border-t border-luxury-gold/20 mt-20">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-12">
        <div className="grid grid-cols-1 md:grid-cols-4 gap-8">
          <div className="space-y-4">
            <div className="flex items-center space-x-2">
              <Crown className="h-8 w-8 text-luxury-gold" />
              <span className="text-2xl font-luxury font-bold text-luxury-gold">
                HeSocial
              </span>
            </div>
            <p className="text-luxury-platinum/80 text-sm leading-relaxed">
              專為高淨值人士打造的頂級社交平台，提供獨家尊榮體驗與精緻社交活動。
            </p>
          </div>

          <div className="space-y-4">
            <h3 className="text-luxury-gold font-luxury font-semibold">服務項目</h3>
            <ul className="space-y-2 text-sm">
              <li>
                <a href="#" className="text-luxury-platinum/80 hover:text-luxury-gold transition-colors">
                  私人晚宴
                </a>
              </li>
              <li>
                <a href="#" className="text-luxury-platinum/80 hover:text-luxury-gold transition-colors">
                  豪華遊艇派對
                </a>
              </li>
              <li>
                <a href="#" className="text-luxury-platinum/80 hover:text-luxury-gold transition-colors">
                  藝術品鑑會
                </a>
              </li>
              <li>
                <a href="#" className="text-luxury-platinum/80 hover:text-luxury-gold transition-colors">
                  商務社交
                </a>
              </li>
            </ul>
          </div>

          <div className="space-y-4">
            <h3 className="text-luxury-gold font-luxury font-semibold">會員專區</h3>
            <ul className="space-y-2 text-sm">
              <li>
                <a href="#" className="text-luxury-platinum/80 hover:text-luxury-gold transition-colors">
                  Platinum 會員
                </a>
              </li>
              <li>
                <a href="#" className="text-luxury-platinum/80 hover:text-luxury-gold transition-colors">
                  Diamond 會員
                </a>
              </li>
              <li>
                <a href="#" className="text-luxury-platinum/80 hover:text-luxury-gold transition-colors">
                  Black Card 會員
                </a>
              </li>
              <li>
                <a href="#" className="text-luxury-platinum/80 hover:text-luxury-gold transition-colors">
                  專屬顧問服務
                </a>
              </li>
            </ul>
          </div>

          <div className="space-y-4">
            <h3 className="text-luxury-gold font-luxury font-semibold">聯絡我們</h3>
            <div className="space-y-3 text-sm">
              <div className="flex items-center space-x-3">
                <Phone className="h-4 w-4 text-luxury-gold" />
                <span className="text-luxury-platinum/80">+886-2-2345-6789</span>
              </div>
              <div className="flex items-center space-x-3">
                <Mail className="h-4 w-4 text-luxury-gold" />
                <span className="text-luxury-platinum/80">concierge@hesocial.com</span>
              </div>
              <div className="flex items-center space-x-3">
                <MapPin className="h-4 w-4 text-luxury-gold" />
                <span className="text-luxury-platinum/80">台北市信義區松仁路</span>
              </div>
            </div>
          </div>
        </div>

        <div className="mt-12 pt-8 border-t border-luxury-gold/20">
          <div className="flex flex-col md:flex-row justify-between items-center">
            <p className="text-luxury-platinum/60 text-sm">
              © 2024 HeSocial. 版權所有 | 隱私政策 | 服務條款
            </p>
            <div className="flex items-center space-x-4 mt-4 md:mt-0">
              <span className="text-luxury-platinum/60 text-sm">
                企業級安全認證
              </span>
              <div className="flex items-center space-x-2">
                <div className="w-2 h-2 bg-green-500 rounded-full"></div>
                <span className="text-luxury-platinum/60 text-sm">系統正常運行</span>
              </div>
            </div>
          </div>
        </div>
      </div>
    </footer>
  )
}

export default Footer
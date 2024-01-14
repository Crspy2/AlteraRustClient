use fuzzywuzzy::fuzz;

use crate::sms::{get_country_prices::CountryPriceInfo, get_service_list::ServiceResponse};

pub fn is_service_blacklisted(service: &str) -> bool {
    let blacklisted_services = vec![
        "5ka.ru",
        "AdvCash",
        "Afterpay",
        "Airtm",
        "Alipay",
        "beCHARGE",
        "Blackcatcard",
        "BluePay",
        "CashApp",
        "CentroBill",
        "Clearpay",
        "CoinsBaron",
        "Coinbase",
        "CoinSwitch",
        "CoinSpot",
        "Coinstash",
        "CornerCard",
        "cPay",
        "Easy Pay",
        "Entropay",
        "ePayments",
        "GCash",
        "Holvi",
        "iCard",
        "iPayYou",
        "Joompay",
        "Leupay",
        "LocalBitcoins",
        "Mezu",
        "MuchBetter",
        "MyBoost",
        "Neteller",
        "OKCoin",
        "Papara",
        "PayGo",
        "Payoneer",
        "Paypal",
        "Paysafe",
        "PaySay",
        "PaySend",
        "Paysera",
        "Paytm",
        "QIWIWallet",
        "SimplexCC",
        "Skrill",
        "Steemit",
        "Stripe",
        "TransferWise",
        "Venmo",
        "Verse",
        "Vimpay",
        "Walmart money card",
        "Webmoney",
        "xcoins",
        "ZapZap",
        "zebpay",
        "ZipCo",
        "ZipPay",
        "Abra",
        "Akulaku",
        "ANZ",
        "Banq24",
        "banxa",
        "BeemIt",
        "Billcom",
        "Bunq",
        "CapitalOne",
        "Cardyard",
        "CashAA",
        "CashZine",
        "CodaPayments",
        "Cogni Bank",
        "CreditKarma",
        "Dukascopy",
        "ecoPayz",
        "eToro",
        "EuroPYM",
        "ExpertOption",
        "FBS",
        "FreshForex",
        "FTX",
        "Go2Bank",
        "GoFundMe",
        "GreenDot",
        "instaforex",
        "InstaGC",
        "Instarem",
        "IQOption",
        "Jerry",
        "JuanCash",
        "KBZpay",
        "KVBPrime",
        "LydiaApp",
        "Monese",
        "Moneylion",
        "MoneyPak",
        "MoneyRawr",
        "Monzo",
        "Naver",
        "NetBank",
        "Netease",
        "NiftyLoans",
        "PayMaya",
        "Paymium",
        "PayQin",
        "Phyre",
        "RazerPay",
        "Remitly",
        "Revolut",
        "Robinhood",
        "SafeCurrency",
        "sharemoney",
        "SwissBorg",
        "SwitchHere",
        "tala",
        "Tenx",
        "The Change",
        "Tikki",
        "ToTalk",
        "TradingView",
        "TransferHome",
        "Trustcom",
        "UBank",
        "Varo",
        "Vodi",
        "Voyager",
        "WestStein",
        "Wing",
        "Wirex",
        "wittix",
        "Womply",
        "X-Bank",
        "Xoom",
        "Yodlee",
        "YouTrip",
        "Zelle",
        "Zest",
        "Indacoin",
        "Coinomi",
        "Coinjar",
    ];

    for s in blacklisted_services {
        if service.eq_ignore_ascii_case(s) {
            return true;
        }
    }

    if service.to_lowercase().contains("coin") {
        return true;
    } else if service.to_lowercase().contains("pay") {
        return true;
    } else if service.to_lowercase().contains("cash") {
        return true;
    } else if service.to_lowercase().contains("sell") {
        return true;
    }

    false
}

pub struct SimilarServiceInfo {
    pub service_info: ServiceResponse,
    pub similarity_score: u8,
}

pub fn find_similar_services(
    desired_service: &str,
    total_services: &Vec<ServiceResponse>,
) -> Vec<SimilarServiceInfo> {
    let mut similar_services: Vec<SimilarServiceInfo> = vec![];
    for service in total_services {
        if is_service_blacklisted(&service.name) {
            continue;
        }
        let similarity_score = fuzz::ratio(
            &desired_service.to_lowercase(),
            &service.name.to_lowercase(),
        );

        if similarity_score == 100 {
            similar_services = vec![SimilarServiceInfo {
                service_info: service.clone(),
                similarity_score,
            }];
            return similar_services;
        } else if similarity_score >= 55 {
            similar_services.append(&mut vec![SimilarServiceInfo {
                service_info: service.clone(),
                similarity_score,
            }]);
        } else if service.name.contains(&desired_service) {
            similar_services.append(&mut vec![SimilarServiceInfo {
                service_info: service.clone(),
                similarity_score,
            }])
        }
    }

    return similar_services;
}

pub struct SimilarCountryInfo {
    pub country_info: CountryPriceInfo,
    pub similarity_score: u8,
}

pub fn find_similar_countries(
    desired_country: &str,
    total_countries: &Vec<CountryPriceInfo>,
) -> Vec<SimilarCountryInfo> {
    let mut similar_countries: Vec<SimilarCountryInfo> = vec![];
    let mut similarity_score: u8;
    for country in total_countries {
        if desired_country.len() <= 3 {
            similarity_score =
                fuzz::ratio(&desired_country.to_lowercase(), &country.iso.to_lowercase());
        } else {
            similarity_score = fuzz::ratio(
                &desired_country.to_lowercase(),
                &country.name.to_lowercase(),
            );
        }

        if similarity_score >= 55 {
            similar_countries.append(&mut vec![SimilarCountryInfo {
                country_info: country.clone(),
                similarity_score,
            }]);
        }
    }

    return similar_countries;
}

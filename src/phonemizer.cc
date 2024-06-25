#include <include/PhonetisaurusScript.h>

class Phonemizer
{
public:
  Phonemizer(std::string &model)
  {
    this->decoder = new PhonetisaurusScript(model, "");
  }

  std::vector<string> phoneticize(std::string &value)
  {
    std::vector<string> results;
    for (auto pathData : this->decoder->Phoneticize(value))
    {
      for (auto symid : pathData.Uniques)
      {
        results.push_back(this->decoder->FindOsym(symid));
      }
    }

    return results;
  }

  std::vector<std::vector<std::string>> phoneticize_sampling(std::string &value, int nbest, int beam, float threshold, double pmass)
  {
    std::vector<std::vector<std::string>> collections;
    for (auto pathData : this->decoder->Phoneticize(value, nbest, beam, threshold, false, false, pmass))
    {
      std::vector<std::string> results;
      for (auto symid : pathData.Uniques)
      {
        results.push_back(this->decoder->FindOsym(symid));
      }
      collections.push_back(results);
    }
    return collections;
  }

  ~Phonemizer()
  {
    if (this->decoder)
    {
      delete this->decoder;
    }
  }

private:
  PhonetisaurusScript *decoder;
};
